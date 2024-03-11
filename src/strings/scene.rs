use core::fmt;
use std::{collections::HashMap, fs};

use actix_web::{
    error::{
        ErrorBadRequest,
        ErrorInternalServerError, ErrorServiceUnavailable
    },
    web,
    HttpRequest
};
use fancy_regex::Regex;
use rand::{distributions::{Distribution, WeightedIndex}, seq::SliceRandom};
use rusqlite::{params, Connection};
use serde::Deserialize;

use super::common;

struct Scene {
    buffer: String,
    data: String,
    data_key: Option<String>,
}

#[derive(Deserialize)]
pub struct NextData {
    value: Option<String>,
}
pub async fn next(req: HttpRequest, info: web::Json<NextData>) -> Result<String, actix_web::Error> {
    // ログインセッションを取得
    let session =  req.cookie("login_session")
        .ok_or(ErrorBadRequest("ログインセッションがありません"))?;
    // データベースに接続
    let conn = common::open_database()?;
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => return Err(ErrorServiceUnavailable(common::SERVER_MAINTENANCE_TEXT)),
            "end" => return Err(ErrorServiceUnavailable(common::SERVER_END_TEXT)),
            _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT))
        }
    }
    // Enoを取得
    let eno = common::session_to_eno(&conn, session.value())?;
    // シーン情報取得
    let scene = conn.query_row(
        "SELECT buffer,data,data_key FROM scene WHERE eno=?1",
        params![eno],
        |row| {
            Ok(Scene {
                buffer: row.get(0)?,
                data: row.get(1)?,
                data_key: row.get(2)?,
            })
        },
    ).map_err(|err| ErrorInternalServerError(err))?;
    // データをHashMapに格納
    let mut data: HashMap<String, String> = serde_json::from_str(&scene.data)
        .unwrap_or(HashMap::new());
    // 渡されたデータを保存
    if let Some(key) = scene.data_key {
        data.insert(key, info.value.to_owned().unwrap_or(String::new()));
    }
    // 文字列を処理
    Ok(
        process_scene(scene.buffer.as_str(), &conn, eno, &mut data)
            .map_err(|err| ErrorInternalServerError(err))?
            .trim_end().to_string()
    )
}

#[derive(Debug)]
enum ScriptError {
    CommandNoOption(String),
    OptionError(String, String),
    NoSaveData(String),
    ParseError(String, String),
    UndefinedCommand,
    SyntaxError,
    RusqliteError(rusqlite::Error),
    CodeError(String),
}
impl ScriptError {
    fn command_no_option(command: &str) -> Self {
        Self::CommandNoOption(command.to_string())
    }
    fn option_error(command: &str, value: &str) -> Self {
        Self::OptionError(command.to_string(), value.to_string())
    }
    fn no_save_data(var: &str) -> Self {
        Self::NoSaveData(var.to_string())
    }
    fn parse_error(value: &str, to: &str) -> Self {
        Self::ParseError(value.to_string(), to.to_string())
    }
}
impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScriptError::CommandNoOption(command) => match command.as_str() {
                "var" => write!(f, "変更する変数名が指定されていない"),
                "location" => write!(f, "移動先のロケーションが指定されていない"),
                "kins" => write!(f, "変更するキンスの額が指定されていない"),
                "fragment" => write!(f, "フラグメントのカテゴリまたはIDが指定されていない"),
                "match" => write!(f, "分岐で参照する変数が指定されていない"),
                _ => write!(f, "不明なコマンドでエラーが発生"),
            },
            ScriptError::OptionError(command, value) => match command.as_str() {
                "fragment" => write!(f, "カテゴリ{}のフラグメントは存在しない", value),
                "if" => write!(f, "指定された条件{}は定義されていない", value),
                _ => write!(f, "不明なコマンドでエラーが発生"),
            },
            ScriptError::NoSaveData(var) => write!(f, "変数\"{}\"が保存されていない", var),
            ScriptError::ParseError(value, to) => write!(f, "\"{}\" は {} に変換できない", value, to),
            ScriptError::UndefinedCommand => write!(f, "定義されていないコマンド"),
            ScriptError::SyntaxError => write!(f, "スクリプトの構文が異常"),
            ScriptError::RusqliteError(err) => err.fmt(f),
            ScriptError::CodeError(err) => err.fmt(f),
        }
    }
}

fn part_option(text: &str) -> (&str, &str) {
    if let Some(pos) = text.find(&[' ', '\n', ';']) {
        if text.bytes().nth(pos) == Some(b';') {
            (&text[..pos], &text[pos + 1..])
        } else {
            (&text[..pos], &text[pos..])
        }
    } else {
        (&text, "")
    }
}
fn get_nest(text: &str) -> Result<(&str, &str, &str), ScriptError> {
    let start = text.find('{').ok_or(ScriptError::SyntaxError)?;
    let prev = text[..start].trim_end();
    let text = text[start + 1..].trim_start();
    let mut pos = 0;
    let mut count = 1;
    let end = loop {
        pos += text[pos..].find(&['{', '}']).ok_or(ScriptError::SyntaxError)?;
        match text.bytes().nth(pos) {
            Some(b'{') => count += 1,
            Some(b'}') => count -= 1,
            _ => return Err(ScriptError::CodeError("ブラケットを正しく取得できない".to_string())),
        }
        if count < 0 {
            return Err(ScriptError::SyntaxError);
        }
        if count == 0 {
            break pos;
        }
        pos += 1;
    };
    Ok((prev, text[..end].trim_end(), &text[end + 1..]))
}

fn process_scene(text: &str, conn: &Connection, eno: i16, data: &mut HashMap<String, String>) -> Result<String, ScriptError> {
    match text.chars().next() {
        // コマンド
        Some('!') => {
            // コマンドと以降の文字列を取得
            let (command, text) = part_option(&text[1..]);
            match command {
                "var" => {
                    // オプションと以降の文字列を取得
                    let (var, text) = part_option(text.trim_start());
                    let (value, text) = part_option(text.trim_start());
                    // 変数を更新
                    if value != "-" {
                        data.insert(var.to_string(), value.to_string());
                    } else {
                        data.remove(&var.to_string());
                    }
                    // 次
                    process_scene(if text.is_empty() {""} else {&text[1..]}, conn, eno, data)
                }
                "location" => {
                    // オプションと以降の文字列を取得
                    let (mut op, mut text) = part_option(text.trim_start());
                    let view = if op == "-" {
                        (op, text) = part_option(text.trim_start());
                        false
                    } else {
                        true
                    };
                    // 移動先を取得
                    let location = match op.chars().next() {
                        // 変数に保存されたもの
                        Some('%') => data.get(&op[1..].to_string()).ok_or(ScriptError::no_save_data(op))?,
                        // 直接指定
                        Some(_) => op,
                        // 指定されていない
                        None => return Err(ScriptError::command_no_option(command))
                    };
                    // ロケーションを変更
                    conn.execute("UPDATE character SET location=?1 WHERE eno=?2", params![location, eno])
                        .map_err(|err| ScriptError::RusqliteError(err))?;
                    if view {
                        // 次
                        Ok(format!("──{}に移動しました", location) + &process_scene(text, conn, eno, data)?)
                    } else {
                        process_scene(if text.is_empty() {""} else {&text[1..]}, conn, eno, data)
                    }
                }
                "kins" => {
                    // オプションと以降の文字列を取得
                    let (value, text) = part_option(text.trim_start());
                    // 変更量を取得
                    let value = value.parse::<i32>().map_err(|_| ScriptError::parse_error(value, "i32"))?;
                    // 現在の所持キンスを取得
                    let kins: i32 = conn.query_row("SELECT kins FROM character WHERE eno=?1", params![eno], |row| row.get(0))
                        .map_err(|err| ScriptError::RusqliteError(err))?;
                    // 変更した際にマイナスになっていないか確認
                    let result = kins + value > 0;
                    // なっていなければ変更
                    if result {
                        conn.execute("UPDATE character SET kins=?1 WHERE eno=?2", params![kins + value, eno])
                            .map_err(|err| ScriptError::RusqliteError(err))?;
                    }
                    // オプションと以降の文字列を取得
                    let (var, text) = part_option(text.trim_start());
                    // 指定が"_"でなかった場合dataに結果を保存
                    if var != "_" {
                        data.insert(var.to_string(), result.to_string());
                    }
                    // 次
                    Ok(format!("──{}キンスを{}", value.abs(), if result {if value > 0 {"入手しました"} else {"支払いました"}} else {"支払えませんでした"}) + &process_scene(text, conn, eno, data)?)
                }
                "fragment" => {
                    // オプションと以降の文字列を取得
                    let (op, text) = part_option(text.trim_start());
                    // 数値に変換できるならID、そうでないならカテゴリ
                    let id = match op.parse::<i32>() {
                        Ok(id) => id,
                        Err(_) => {
                            let fragments = match op {
                                "all" => {
                                    let mut stmt = conn.prepare("SELECT id FROM base_fragment WHERE category NOT IN ('記念','世界観','秘匿','名前')")
                                        .map_err(|err| ScriptError::RusqliteError(err))?;
                                    let x = stmt
                                        .query_map([], |row| row.get(0))
                                        .map_err(|err| ScriptError::RusqliteError(err))?
                                        .collect::<Result<Vec<i32>, rusqlite::Error>>()
                                        .map_err(|err| ScriptError::RusqliteError(err))?;
                                    x
                                }
                                _ => {
                                    let mut stmt = conn.prepare("SELECT id FROM base_fragment WHERE category=?1")
                                        .map_err(|err| ScriptError::RusqliteError(err))?;
                                    let x = stmt
                                        .query_map(params![op], |row| row.get(0))
                                        .map_err(|err| ScriptError::RusqliteError(err))?
                                        .collect::<Result<Vec<i32>, rusqlite::Error>>()
                                        .map_err(|err| ScriptError::RusqliteError(err))?;
                                    x
                                }
                            };
                            *fragments.choose(&mut rand::thread_rng())
                                .ok_or(ScriptError::option_error("fragment", op))?
                        }
                    };
                    // idからフラグメントを取得
                    let fragment = conn.query_row(
                        "SELECT category,name,lore,status,skill FROM base_fragment WHERE id=?1",
                        params![id],
                        |row| Ok(common::Fragment::new(
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?
                        )),
                    ).map_err(|err| ScriptError::RusqliteError(err))?;
                    // 獲得処理
                    let get = common::add_fragment(&conn, eno, &fragment)
                        .map_err(|err| ScriptError::RusqliteError(err))?;
                    // 次
                    Ok(format!("──フラグメント『{}』を入手しました{}", fragment.name, if !get {"<br>──所持数制限により破棄されました"} else {""}) + &process_scene(text, conn, eno, data)?)
                }
                "name" => {
                    // オプションと以降の文字列を取得
                    let (op, text) = part_option(text.trim_start());
                    // 名前を取得
                    let name = match op.chars().next() {
                        // 変数に保存されたもの
                        Some('%') => data.get(&op[1..].to_string()).ok_or(ScriptError::no_save_data(op))?,
                        // 直接指定
                        Some(_) => op,
                        // 指定されていない
                        None => ""
                    };
                    // フラグメントデータを作成
                    let fragment = match name {
                        "" => common::Fragment::new(
                            "名前".to_string(),
                            "名無し".to_string(),
                            "あなたは名乗らない。<br>それは自らの選択か、あるいは名乗る名が無いのか。".to_string(),
                            [0, 15, 0, 6, 0, 1, 0, 1],
                            None
                        ),
                        _ => common::Fragment::new(
                            "名前".to_string(),
                            name.to_string(),
                            "あなたの名前。<br>決して無くさないよう、零さないよう。".to_string(),
                            [0, 15, 0, 10, 0, 0, 0, 0],
                            None
                        ),
                    };
                    // 獲得処理
                    let get = common::add_fragment(&conn, eno, &fragment)
                        .map_err(|err| ScriptError::RusqliteError(err))?;
                    // 次
                    Ok(format!("──フラグメント『{}』を入手しました{}", fragment.name, if get {"<br>──所持数制限により破棄されました"} else {""}) + &process_scene(text, conn, eno, data)?)
                }
                // "battle" => {
                //     // !battle (id)
                //     // id以外に指定しなきゃいけないことある？　多分ない……
                //     // 指定されたIDのnpcと戦闘を行う
                //     // オプションと以降の文字列を取得
                //     let (id, text) = part_option(text.trim_start());
                //     // idを取得
                //     let id = id.parse::<u8>().map_err(|_| ScriptError::parse_error(id, "u8"))?;
                //     // 戦闘処理
                //     let log = battle::battle([eno, id as i16 * -1]).map_err(|err| ScriptError::CodeError(err))?;
                    
                //     let log_text = serde_json::to_string(&log).map_err(|err| ScriptError::CodeError(err.to_string()))?;
                //     todo!()
                // }
                "match" => {
                    let (var, mut nest, text) = get_nest(text.trim_start())?;
                    if let Some(nest) = if var == "%" {
                        // 確率
                        let content = rand::random::<u8>();
                        // 条件に合致するキーのネストを取得
                        loop {
                            if nest == "" {
                                break None;
                            }
                            let (key, value, next) = get_nest(nest)?;
                            let key = key.parse::<u8>().map_err(|_| ScriptError::parse_error(key, "u8"))?;
                            // 255を指定した際に確実に通るように　この場合0を指定しても1/255の確率で通るが、実用上通らないネストに意味は無いので別にいい
                            if key >= content {
                                break Some(value);
                            }
                            nest = next.trim_start();
                        }
                    } else {
                        // 変数から判定対象の文字列を取得
                        let content = data.get(&var.to_string()).map(|x| x.as_str());
                        // 条件に合致するキーのネストを取得
                        loop {
                            if nest == "" {
                                break None;
                            }
                            let (key, value, next) = get_nest(nest)?;
                            if Some(key) == content || key == "_" {
                                break Some(value);
                            }
                            nest = next.trim_start();
                        }
                    } {
                        // 中身と次のテキストを結合して処理
                        process_scene(&(nest.to_string() + text), conn, eno, data)
                    } else {
                        // 該当するネストが無ければ行を潰す
                        process_scene(if text.is_empty() {""} else {&text[1..]}, conn, eno, data)
                    }
                }
                "if" => {
                    let (predicate, nest, text) = get_nest(text.trim_start())?;
                    if let Some(nest) = match predicate {
                        // 名前が無ければ
                        "lost_name" => {
                            // 名前を取得
                            let name = match conn.query_row(
                                "SELECT name FROM fragment WHERE eno=?1 AND category='名前' LIMIT 1",
                                params![eno],
                                |row| row.get(0)
                            ) {
                                Ok(name) => Some(name),
                                Err(err) => match err {
                                    rusqlite::Error::QueryReturnedNoRows => None,
                                    _ => return Err(ScriptError::RusqliteError(err)),
                                }
                            };
                            // 名前を取得できたか確認
                            if let Some(name) = name {
                                // 名前がある（保存する）
                                data.insert("name".to_string(), name);
                                // 条件を満たさない
                                None
                            } else {
                                // 条件を満たす
                                Some(nest)
                            }
                        }
                        _ => return Err(ScriptError::option_error("if", predicate)),
                    } {
                        // 条件を満たす
                        process_scene(&(nest.to_string() + text), conn, eno, data)
                    } else {
                        // 該当するネストが無ければ行を潰す
                        process_scene(if text.is_empty() {""} else {&text[1..]}, conn, eno, data)
                    }
                }
                "return" => {
                    // 以降を捨てて次
                    process_scene("", conn, eno, data)
                }
                "yield" => {
                    // オプションと以降の文字列を取得
                    let (var, text) = part_option(text.trim_start());
                    let var = if var != "_" && var != "" {
                        Some(var)
                    } else {
                        None
                    };
                    // シーンを保存
                    conn.execute("UPDATE scene SET buffer=?1,data=?2,data_key=?3 WHERE eno=?4", params![
                        if text.is_empty() {""} else {&text[1..]},
                        serde_json::to_string(&data).map_err(|_| ScriptError::parse_error("data", "String"))?,
                        var,
                        eno,
                    ]).map_err(|err| ScriptError::RusqliteError(err))?;
                    // 終了
                    Ok(String::new())
                }
                _ => Err(ScriptError::UndefinedCommand)
            }
        }
        // 変数
        Some('%') => {
            let end = text[1..].find('%')
                .ok_or(ScriptError::SyntaxError)?;
            let var = &text[1..end + 1];
            let content = data.get(&var.to_string())
                .ok_or(ScriptError::no_save_data(var))?;
            Ok(content.to_owned() + &process_scene(&text[end + 2..], conn, eno, data)?)
        }
        // 通常文字列
        Some(_) => {
            // テキストを取得
            let display = if let Some(next) = text.find(&['!', '%']) {
                // 次にコマンドがあれば、ここまでの情報と次
                text[..next].to_string() + &process_scene(&text[next..], conn, eno, data)?
            } else {
                // 無ければ、ファイル最後の改行を削除して次
                text.trim_end().to_string() + &process_scene("", conn, eno, data)?
            };
            // 表示用文字列を保存
            conn.execute("UPDATE scene SET display=?1 WHERE eno=?2", params![
                display.trim_end(),
                eno,
            ]).map_err(|err| ScriptError::RusqliteError(err))?;
            // 終了
            Ok(display)
        }
        // 文字列が空
        None => {
            // ロケーションを取得
            let location: String = conn.query_row("SELECT location FROM character WHERE eno=?1", params![eno], |row| row.get(0))
                .map_err(|err| ScriptError::RusqliteError(err))?;
            // 現在のロケーションに対応したシーンのリストを取得
            let mut stmt = conn.prepare("SELECT name,weight FROM scene_list WHERE location=?1")
                .map_err(|err| ScriptError::RusqliteError(err))?;
            let scenes: Vec<(String, i32)> = stmt
                .query_map(params![location], |row| Ok((row.get(0)?, row.get(1)?)))
                .map_err(|err| ScriptError::RusqliteError(err))?
                .collect::<Result<_, _>>()
                .map_err(|err| ScriptError::RusqliteError(err))?;
            // 次のテキストを取得、整形
            let text = if scenes.iter().count() != 0 {
                // ランダム抽選の用意
                let weight = WeightedIndex::new(scenes.iter().map(|x| x.1).collect::<Vec<_>>())
                    .map_err(|err| ScriptError::CodeError(err.to_string()))?;
                // ランダムに決定されたファイルからテキストを取得
                fs::read_to_string(format!("game/scene/{}", &scenes[weight.sample(&mut rand::thread_rng())].0))
            } else {
                // そのロケーションにイベントが一件も登録されていなければ、デフォルト
                fs::read_to_string("game/scene/何もない")
            }.map_err(|err| ScriptError::CodeError(err.to_string()))?
                .replace("\t", "")
                .replace("\r\n", "\n")
                .replace("\r", "\n")
                .replace("\\\n", "<br>");
            // スクリプトコメントの正規表現
            let re = Regex::new("##.*\n")
                .map_err(|err| ScriptError::CodeError(err.to_string()))?;
            // スクリプトコメントを削除し次を開始
            process_scene(&("\n<br>……\n…………\n<br>".to_string() + &re.replace_all(&text, "")), conn, eno, data)
        }
    }
}