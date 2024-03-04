use std::{collections::HashMap, fs};

use actix_web::{
    error::{
        ErrorBadRequest,
        ErrorInternalServerError, ErrorServiceUnavailable
    },
    web,
    HttpRequest
};
use rand::{distributions::{Distribution, WeightedIndex}, seq::SliceRandom, thread_rng};
use rusqlite::params;
use serde::Deserialize;

use super::common;

struct Scene {
    buffer: String,
    data: String,
    data_key: String,
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
    let mut data: HashMap<String, String> = if scene.data != "" {
        serde_json::from_str(&scene.data)
            .map_err(|err| ErrorInternalServerError(err))?
    } else { HashMap::new() };
    // 渡されたデータを保存
    if scene.data_key != "" {
        data.insert(scene.data_key, info.value.clone().unwrap_or(String::new()));
    }
    // 文字列を処理
    process_line(scene.buffer.as_str(), eno, &mut data)
}

fn script_error(caption: &str) -> actix_web::Error {
    ErrorInternalServerError("スクリプトエラー : ".to_string() + caption)
}

fn get_option(text: &str) -> (Vec<&str>, Option<usize>) {
    let option_end_pos = text.find('\n');
    (if let Some(pos) = option_end_pos { &text[..pos] } else { text }.trim().split(' ').collect(), option_end_pos)
}

fn get_nest(text: &str) -> Result<(&str, usize), actix_web::Error> {
    if text.chars().nth(0) != Some('{') {
        return Err(script_error("ネストの範囲が正しく指定されていません"));
    }
    let mut index = 1 as usize;
    let mut count = 1i8;
    while count != 0 {
        match text[index..].find(&['{', '}']) {
            Some(pos) => {
                match text[index..].bytes().nth(pos) {
                    Some(b'{') => count += 1,
                    Some(b'}') => count -= 1,
                    _ => return Err(script_error("ネスト処理中の不明なエラーです")),
                }
                index += pos + 1;
            }
            // 中括弧がこれ以上見つからないのに数の整合が合っていない場合、構文エラー
            None => if count > 0 {
                return Err(script_error("ネストが閉じられていません"));
            }
        }
        if count < 0 {
            // 構文エラー
            return Err(script_error("対応しない位置でネストが閉じられようとしています"));
        }
    }
    Ok((&text[1..index - 1].trim(), index))
}

pub fn process_line(scene: &str, eno: i16, data: &mut HashMap<String, String>) -> Result<String, actix_web::Error> {
    match scene.chars().nth(0) {
        // コマンド
        Some('!') => {
            let command_end_pos = scene[1..].find(&[' ', '\n']);
            let (command, scene) = if let Some(pos) = command_end_pos {
                (scene[1..pos + 1].trim_end(), &scene[pos + 2..])
            } else {
                (&scene[1..], "")
            };
            match command {
                // 中断
                "yield" => {
                    let (option, option_end_pos) = get_option(scene);
                    // シーンを保存
                    let conn = common::open_database()?;
                    conn.execute("UPDATE scene SET buffer=?1,data=?2,data_key=?3 WHERE eno=?4", params![
                        if let Some(pos) = option_end_pos { scene[pos..].trim_start() } else { scene },
                        serde_json::to_string(&data).map_err(|err| ErrorInternalServerError(err))?,
                        if let Some(key) = option.get(0) { key } else { "" },
                        eno,
                    ]).map_err(|err| ErrorInternalServerError(err))?;
                    // 終了
                    Ok(String::new())
                }
                // ロケーションを変更
                "location" => {
                    let (option, option_end_pos) = get_option(scene);
                    // ロケーションを保存
                    let conn = common::open_database()?;
                    let key = option.get(0).ok_or(script_error("変数が指定されていません"))?;
                    let location = if key.chars().nth(0) == Some('!') {
                        &key[1..]
                    } else {
                        &data.get(&key.to_string()).ok_or(script_error("変数が保存されていません"))?
                    };
                    conn.execute("UPDATE character SET location=?1 WHERE eno=?2", params![location, eno])
                        .map_err(|err| ErrorInternalServerError(err))?;
                    // まだシーンが終了していなければ継続
                    if let Some(pos) = option_end_pos {
                        process_line(&scene[pos..], eno, data)
                    } else {
                        // 終了
                        Ok(String::new())
                    }
                }
                // フラグメントを獲得
                "fragment" => {
                    let (option, option_end_pos) = get_option(scene);
                    let key = option.get(0)
                        .ok_or(script_error("フラグメントが指定されていません"))?;
                    // データベース接続
                    let conn = common::open_database()?;
                    let fragment = if *key == "名前" {
                        let name = data.get(option.get(1).ok_or(script_error("変数が指定されていません"))?.to_owned()).ok_or(script_error("変数が保存されていません"))?.to_string();
                        if name == "" {
                            common::Fragment::new(
                                "名前".to_string(),
                                "名無し".to_string(),
                                "あなたは名乗らない。<br>それは自らの選択か、あるいは名乗る名が無いのか。".to_string(),
                                [0, 15, 0, 6, 0, 1, 0, 1],
                                None
                            )
                        } else {
                            common::Fragment::new(
                                "名前".to_string(),
                                name,
                                "あなたの名前。<br>決して無くさないよう、零さないよう。".to_string(),
                                [0, 15, 0, 10, 0, 0, 0, 0],
                                None
                            )
                        }
                    } else {
                        // idを取得
                        let id = match key.parse::<i32>() {
                            // idの形式で指定
                            Ok(id) => id,
                            // カテゴリの形式で指定
                            Err(_) => {
                                let result = if *key == "all" {
                                    let mut stmt = conn.prepare("SELECT id FROM base_fragment")
                                        .map_err(|err| ErrorInternalServerError(err))?;
                                    let x = stmt
                                        .query_map([], |row| Ok(row.get(0)?))
                                        .map_err(|err| ErrorInternalServerError(err))?
                                        .collect::<Result<Vec<i32>, rusqlite::Error>>()
                                        .map_err(|err| ErrorInternalServerError(err))?;
                                    x
                                } else {
                                    let mut stmt = conn.prepare("SELECT id FROM base_fragment WHERE category=?1")
                                        .map_err(|err| ErrorInternalServerError(err))?;
                                    let x = stmt
                                        .query_map(params![key], |row| Ok(row.get(0)?))
                                        .map_err(|err| ErrorInternalServerError(err))?
                                        .collect::<Result<Vec<i32>, rusqlite::Error>>()
                                        .map_err(|err| ErrorInternalServerError(err))?;
                                    x
                                };
                                let mut rng = thread_rng();
                                *result.choose(&mut rng)
                                    .ok_or(script_error("指定されたカテゴリのフラグメントは存在しません"))?
                            }
                        };
                        // idからフラグメントを取得
                        conn.query_row(
                            "SELECT category,name,lore,status,skill FROM base_fragment WHERE id=?1",
                            params![id],
                            |row| Ok(common::Fragment::new(
                                row.get(0)?,
                                row.get(1)?,
                                row.get(2)?,
                                row.get(3)?,
                                row.get(4)?
                            )),
                        ).map_err(|err| ErrorInternalServerError(err))?
                    };
                    let get = common::add_fragment(&conn, eno, &fragment)
                        .map_err(|err| ErrorInternalServerError(err))?;
                    // まだシーンが終了していなければ継続
                    if let Some(pos) = option_end_pos {
                        Ok(format!("\n──フラグメント『{}』を入手しました{}", fragment.name, if get {""} else {"<br>──所持数制限により破棄されました"}) + &process_line(&scene[pos..], eno, data)?)
                    } else {
                        // 終了
                        Ok(String::new())
                    }
                }
                //  キンス変化
                "kins" => {
                    let (option, option_end_pos) = get_option(scene);
                    // 値を取得
                    let value: i32 = option.get(0)
                        .ok_or(script_error("額が指定されていません"))?
                        .parse()
                        .map_err(|_| script_error("額を正しく認識できません"))?;
                    // データベース接続
                    let conn = common::open_database()?;
                    // 現在額を取得
                    let kins: i32 = conn.query_row("SELECT kins FROM character WHERE eno=?1", params![eno], |row| Ok(row.get(0)?))
                        .map_err(|err| ErrorInternalServerError(err))?;
                    // 変更額が正の数、または現在の所持キンスから変更額だけ減らしても0未満にならないなら変更
                    let result = if value >= 0 || kins >= value {
                        conn.execute("UPDATE character SET kins=kins+?1 WHERE eno=?2", params![value, eno])
                            .map_err(|err| ErrorInternalServerError(err))?;
                        // データ更新用文字列
                        "ok"
                    } else { "err" };
                    // もしデータキーが指定されていれば
                    if let Some(data_key) = option.get(1) {
                        // データを保存
                        data.insert(data_key.to_string(), result.to_string());
                        // データベース側にも保存（これ本当にこのタイミングじゃないといけないかちょっと分からない）
                        conn.execute("UPDATE scene SET data=?1 WHERE eno=?2", params![
                            serde_json::to_string(&data).map_err(|err| ErrorInternalServerError(err))?,
                            eno,
                        ]).map_err(|err| ErrorInternalServerError(err))?;
                    }
                    // まだシーンが終了していなければ継続
                    if let Some(pos) = option_end_pos {
                        Ok(format!("\n──{}キンスを{}", value.abs(), if value < 0 {"支払いました"} else {"入手しました"}) + &process_line(&scene[pos..], eno, data)?)
                    } else {
                        // 終了
                        Ok(String::new())
                    }
                }
                // 変数設定
                "var" => {
                    let (option, option_end_pos) = get_option(scene);
                    // 値を取得
                    let data_key = option.get(0)
                        .ok_or(script_error("変数が指定されていません"))?;
                    if let Some(value) = option.get(1) {
                        // データを保存
                        data.insert(data_key.to_string(), value.to_string());
                    } else {
                        // データを削除
                        data.remove(&data_key.to_string());
                    }
                    // データベース接続
                    let conn = common::open_database()?;
                    // データベース側にも保存
                    conn.execute("UPDATE scene SET data=?1 WHERE eno=?2", params![
                        serde_json::to_string(&data).map_err(|err| ErrorInternalServerError(err))?,
                        eno,
                    ]).map_err(|err| ErrorInternalServerError(err))?;
                    // まだシーンが終了していなければ継続
                    if let Some(pos) = option_end_pos {
                        process_line(&scene[pos..], eno, data)
                    } else {
                        // 終了
                        Ok(String::new())
                    }
                }
                // 終了
                "return" => {
                    let conn = common::open_database()?;
                    conn.execute("UPDATE scene SET buffer='',display=?1,data=?2,data_key='' WHERE eno=?3", params![
                        scene,
                        serde_json::to_string(&data).map_err(|err| ErrorInternalServerError(err))?,
                        eno,
                    ]).map_err(|err| ErrorInternalServerError(err))?;
                    Ok(String::new())
                }
                // 分岐
                "match" => {
                    let nest_start_pos = scene.find('{')
                        .ok_or(script_error("分岐の構文が異常です"))?;
                    let key = scene[..nest_start_pos].trim();
                    let (mut nest, nest_end_pos) = get_nest(&scene[nest_start_pos..])?;
                    let mut content = if key == "%" {
                        let check = rand::random::<u8>();
                        loop {
                            match nest.find('{') {
                                Some(tag_end_pos) => {
                                    let (content, content_end_pos) = get_nest(&nest[tag_end_pos..])?;
                                    let part = nest[..tag_end_pos].trim().parse::<u8>().map_err(|_| script_error("確率分岐のキーが数値ではありません"))?;
                                    if part >= check {
                                        break content;
                                    }
                                    nest = &nest[tag_end_pos + content_end_pos..];
                                }
                                None => break "",
                            }
                        }
                    } else {
                        let check = match data.get(key) {
                            Some(value) => value.as_str(),
                            None => "",
                        };
                        loop {
                            match nest.find('{') {
                                Some(tag_end_pos) => {
                                    let (content, content_end_pos) = get_nest(&nest[tag_end_pos..])?;
                                    let part = nest[..tag_end_pos].trim();
                                    if part == "_" || part == check {
                                        break content;
                                    }
                                    nest = &nest[tag_end_pos + content_end_pos..];
                                }
                                None => break "",
                            }
                        }
                    }.to_string();
                    content += &scene[nest_start_pos + nest_end_pos..];
                    Ok(process_line(&content, eno, data)?.to_string())
                }
                "if" => {
                    let nest_start_pos = scene.find('{')
                        .ok_or(script_error("分岐の構文が異常です"))?;
                    let key = scene[..nest_start_pos].trim();
                    let (nest, nest_end_pos) = get_nest(&scene[nest_start_pos..])?;
                    match key {
                        "lost_name" => {
                            // データベース接続
                            let conn = common::open_database()?;
                            // 名前を取得
                            let name: Option<String> = match conn.query_row(
                                "SELECT name FROM fragment WHERE eno=?1 AND category='名前' LIMIT 1",
                                params![eno],
                                |row| Ok(row.get(0)?)
                            ) {
                                Ok(name) => Some(name),
                                Err(err) => match err {
                                    rusqlite::Error::QueryReturnedNoRows => None,
                                    _ => return Err(ErrorInternalServerError(err)),
                                }
                            };
                            // 名前を取得できたか確認
                            if let Some(name) = name {
                                // 名前がある（保存する）
                                data.insert("name".to_string(), name);
                                conn.execute("UPDATE scene SET data=?1 WHERE eno=?2", params![
                                    serde_json::to_string(&data).map_err(|err| ErrorInternalServerError(err))?,
                                    eno,
                                ]).map_err(|err| ErrorInternalServerError(err))?;
                                // ネストの終了地点から
                                Ok(process_line(&scene[nest_start_pos + nest_end_pos..].trim_start(), eno, data)?.to_string())
                            } else {
                                // 名前がない（ネストの中身を処理）
                                Ok(process_line(&format!("{}{}", nest, &scene[nest_start_pos + nest_end_pos..]), eno, data)?.to_string())
                            }
                        }
                        _ => Err(script_error("定義されていないコマンドです"))
                    }
                }
                _ => Err(script_error("定義されていないコマンドです"))
            }
        }
        // 変数出力
        Some('%') => {
            let end = scene[1..].find('%').ok_or(script_error("変数名が異常です"))?;
            let text = data.get(&scene[1..end + 1].to_string()).ok_or(script_error("変数が保存されていません"))?;
            Ok(text.clone() + &process_line(&scene[end + 2..], eno, data)?)
        }
        // コメント
        Some('#') => {
            // 行末までを取得
            match scene.find('\n') {
                // 次がある
                Some(end) => {
                    process_line(&scene[end..], eno, data)
                }
                // これで終了
                None => {
                    let conn = common::open_database()?;
                    conn.execute("UPDATE scene SET buffer='',display=?1,data=?2,data_key='' WHERE eno=?3", params![
                        scene,
                        serde_json::to_string(&data).map_err(|err| ErrorInternalServerError(err))?,
                        eno,
                    ]).map_err(|err| ErrorInternalServerError(err))?;
                    Ok(String::new())
                }
            }
        }
        // 通常文字列
        Some(_) => {
            // シーンを保存
            let conn = common::open_database()?;
            match scene.find(&['!', '%', '#']) {
                // 次のコマンドがある
                Some(pos) => {
                    conn.execute("UPDATE scene SET buffer='',display=?1,data=?2,data_key='' WHERE eno=?3", params![
                        &scene[..pos].trim_end(),
                        serde_json::to_string(&data).map_err(|err| ErrorInternalServerError(err))?,
                        eno,
                    ]).map_err(|err| ErrorInternalServerError(err))?;
                    Ok(String::from(scene[..pos].trim_end()) + &process_line(&scene[pos..], eno, data)?)
                }
                // これで終了"
                None => {
                    conn.execute("UPDATE scene SET buffer='',display=?1,data=?2,data_key='' WHERE eno=?3", params![
                        scene,
                        serde_json::to_string(&data).map_err(|err| ErrorInternalServerError(err))?,
                        eno,
                    ]).map_err(|err| ErrorInternalServerError(err))?;
                    Ok(String::from(scene))
                }
            }
        }
        // 文字列が空
        None => {
            // 現在のロケーションを取得
            let conn = common::open_database()?;
            let location = conn.query_row(
                "SELECT location FROM character WHERE eno=?1",
                params![eno],
                |row| row.get::<usize, String>(0),
            ).map_err(|err| ErrorInternalServerError(err))?;
            // 現在のロケーションに対応したシーンのリストを取得
            let mut stmt = conn.prepare("SELECT name,weight FROM scene_list WHERE location=?1")
                .map_err(|err| ErrorInternalServerError(err))?;
            // シーンリストの取得（ファイルパス, 重み）
            let scenes = stmt
                .query_map(params![location], |row| Ok((row.get::<usize, String>(0)?, row.get::<usize, i32>(1)?)))
                .map_err(|err| ErrorInternalServerError(err))?
                .collect::<Result<Vec<(String, i32)>, rusqlite::Error>>()
                .map_err(|err| ErrorInternalServerError(err))?;
            if scenes.iter().count() == 0 {
                return Ok(process_line(fs::read_to_string("game/scene/何もない").unwrap().as_str(), eno, data)?.to_string())
            }
            // ランダム抽選の用意
            let dist = WeightedIndex::new(scenes.iter().map(|x| x.1).collect::<Vec<i32>>())
                .map_err(|err| actix_web::error::ErrorInternalServerError(err))?;
            let mut rng = rand::thread_rng();
            // ランダムに決定されたファイルからテキストを取得
            let scene =
                fs::read_to_string("game/scene/".to_string() + &scenes[dist.sample(&mut rng)].0)
                .map_err(|err| ErrorInternalServerError(err))?;
            // インデントを消去して処理開始
            Ok(process_line(scene.replace('\t', "").as_str(), eno, data)?.to_string())
        }
    }
}
