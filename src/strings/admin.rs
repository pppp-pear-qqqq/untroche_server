use std::fs;

use actix_web::{error::{ErrorBadRequest, ErrorForbidden, ErrorInternalServerError}, web, HttpRequest, HttpResponse};
use awc::cookie::Cookie;
use fancy_regex::Regex;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use super::{
    battle,
    common,
};

// データベースに保存されているパスワードを取得
pub(super) fn check_server_password(conn: &Connection, password: &str) -> Result<(), actix_web::Error> {
    if password == conn.query_row("SELECT password FROM server", [], |row| {
        row.get::<usize, String>(0)
    }).map_err(|err| ErrorInternalServerError(err))? {
        Ok(())
    } else {
        Err(ErrorForbidden("パスワードが違います"))
    }
}

// めちゃめちゃ雑にsql文を実行する
// return 成否: bool
#[allow(dead_code)]
fn sql_execute(conn: &Connection, sql: &str) -> bool {
    match conn.execute(sql, []) {
        Ok(line) => {
            println!("successed: {} : {}", sql, line);
            true
        }
        Err(err) => {
            println!("failed: {} : {}", sql, err);
            false
        }
    }
}

#[derive(Serialize, Deserialize)]
struct NPCWord {
    start: Option<String>,
    win: Option<String>,
    lose: Option<String>,
    draw: Option<String>,
    escape: Option<String>,
}
#[allow(dead_code)]
impl NPCWord {
    fn new(start: Option<&str>, win: Option<&str>, lose: Option<&str>, draw: Option<&str>, escape: Option<&str>) -> Self {
        Self {
            start: start.map(|x| x.to_string()),
            win: win.map(|x| x.to_string()),
            lose: lose.map(|x| x.to_string()),
            draw: draw.map(|x| x.to_string()),
            escape: escape.map(|x| x.to_string()),
        }
    }
}

// サーバー起動時に実行する関数
#[allow(dead_code)]
pub fn preset() -> Result<(), rusqlite::Error> {
    // let conn = Connection::open(common::DATABASE)?;
    // sql_execute(&conn, "CREATE TABLE npc(id INTEGER NOT NULL PRIMARY KEY,name TEXT NOT NULL,acronym TEXT NOT NULL,color BLOB NOT NULL,word TEXT NOT NULL,status BLOB NOT NULL)");
    // sql_execute(&conn, "CREATE TABLE npc_skill(id INTEGER NOT NULL,slot INTEGER NOT NULL,skill INTEGER NOT NULL,name TEXT,word TEXT,PRIMARY KEY(id,slot))");
    // sql_execute(&conn, "CREATE TABLE reward(id INTEGER NOT NULL PRIMARY KEY,npc INTEGER NOT NULL,weight INTEGER NOT NULL,category TEXT NOT NULL,name TEXT NOT NULL,lore TEXT NOT NULL,status BLOB NOT NULL,skill INTEGER)");
    // sql_execute(&conn, "DELETE FROM npc");
    // if let Err(err) = conn.execute("INSERT INTO npc(name,acronym,color,word,status) VALUES(?1,?2,?3,?4,?5)", params![
    //     "盗賊女",
    //     "賊",
    //     [176u8, 54u8, 78u8],
    //     serde_json::to_string(&NPCWord::new(
    //         Some("「さっさとくたばってね」"),
    //         Some("「はっ。あたしの勝ちだ」"),
    //         Some("「……ぐっ、くそッ！」"),
    //         Some("「……今日は、この辺にしといてやる」"),
    //         Some("「ち。逃がしたか」"),
    //     )).unwrap(),
    //     [0u8, 29, 0, 9, 0, 18, 0, 11],
    // ]) { println!("{}", err) }
    // sql_execute(&conn, "DELETE FROM npc_skill");
    // if let Err(err) = conn.execute("INSERT INTO npc_skill VALUES(1,1,?1,?2,?3),(1,2,?4,?5,?6),(1,3,?7,?8,?9),(1,4,?10,?11,?12),(1,5,?13,?14,?15)", params![
    //     4, Option::<&str>::None, Some("引掻き傷。"),
    //     24, Option::<&str>::None, Some("「逃がさないよ」"),
    //     16, Option::<&str>::None, Option::<&str>::None,
    //     80, Option::<&str>::None, Some("「──チッ」"),
    //     23, Option::<&str>::None, Some("「もっと近くに来なよ、爪が届かない」"),
    // ]) { println!("{}", err) }
    // if let Err(err) = conn.execute("INSERT INTO reward(npc,weight,category,name,lore,status,skill) VALUES(?1,?2,?3,?4,?5,?6,?7)", params![1, 1,
    //     "クラック", "テストフラグメント", "戦闘報酬のテスト", [0u8, 0, 0, 0, 0, 0, 0, 0], Option::<i32>::None,
    // ]) { println!("{}", err) }
    // sql_execute(&conn, "DELETE FROM user");
    // sql_execute(&conn, "DELETE FROM character");
    // sql_execute(&conn, "DELETE FROM character_profile");
    // sql_execute(&conn, "DELETE FROM scene");
    // sql_execute(&conn, "DELETE FROM fragment");
    // sql_execute(&conn, "DELETE FROM login_session");
    // sql_execute(&conn, "DELETE FROM battle_reserve");
    // sql_execute(&conn, "DELETE FROM battle");
    // sql_execute(&conn, "DELETE FROM timeline");
    // sql_execute(&conn, "INSERT INTO fragment SELECT 1,id-25,category,name,lore,status,skill,NULL,NULL FROM base_fragment WHERE id>30 AND id<=55");
    // sql_execute(&conn, "INSERT INTO fragment SELECT 2,id-50,category,name,lore,status,skill,NULL,NULL FROM base_fragment WHERE id>55 AND id<=80");
    // sql_execute(&conn, "INSERT INTO scene_list VALUES('海辺/何もない','海辺',1),('海辺/所持限界','海辺',1),('海辺/釣り','海辺',1),('海辺/宝探し','海辺',1),('森林/果樹','森林',1),('森林/花の香','森林',1),('森林/隙間から','森林',1),('森林/丸太小屋','森林',1),('草原/装備品2','草原',1),('岩場/尖塔を眺める','岩場',1),('花の広場','花の広場',1)");
    // let mut stmt = conn.prepare("SELECT f.id,f.category,f.name,f.lore,f.status,f.skill,s.name,s.lore,s.type,s.effect FROM base_fragment f LEFT OUTER JOIN skill s ON f.skill=s.id")?;
    // let result = stmt.query_map([], |row| {
    //     let skill: Option<i32> = row.get(5)?;
    //     if let Some(skill) = skill {
    //         Ok((
    //             row.get(0)?,
    //             row.get(1)?,
    //             row.get(2)?,
    //             row.get(3)?,
    //             row.get(4)?,
    //             Some(skill),
    //             Some(row.get(6)?),
    //             Some(row.get(7)?),
    //             Some(battle::Timing::from(row.get(8)?)),
    //             Some(battle::Command::convert(row.get(9)?).map_err(|_| rusqlite::Error::QueryReturnedNoRows)?),
    //         ))
    //     } else {
    //         Ok((
    //             row.get(0)?,
    //             row.get(1)?,
    //             row.get(2)?,
    //             row.get(3)?,
    //             row.get(4)?,
    //             None, None, None, None, None,
    //         ))
    //     }
    // })?.collect::<Result<Vec<(i32, String, String, String, [u8; 8], Option<i32>, Option<String>, Option<String>, _, _)>, _>>()?;
    // for r in result {
    //     println!("================\nid{} {} {} {:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}\n{}", r.0, r.1, r.2, r.4[0], r.4[1], r.4[2], r.4[3], r.4[4], r.4[5], r.4[6], r.4[7], r.3);
    //     if let Some(skill) = r.5 {
    //         println!("----------------\nid{} {} ({})\n{}", skill, r.6.unwrap(), r.8.unwrap().to_i8(), r.7.unwrap());
    //     }
    // }
    Ok(())
}

#[derive(Deserialize)]
pub(super) struct Password {
    pass: String,
}
// 管理者用ページ
pub(super) async fn index(pass: web::Query<Password>) -> Result<HttpResponse, actix_web::Error> {
	// データベースに接続
	let conn = common::open_database()?;
    // URLに含まれるパスワード部分を取得して確認
    check_server_password(&conn, &pass.pass)?;
    // パスワードが一致していればいい感じのを返す
    || -> Result<HttpResponse, liquid::Error> {
        Ok(HttpResponse::Ok()
            .cookie(Cookie::build("admin_password", &pass.pass)
                .finish()
            ).body(
                liquid::ParserBuilder::with_stdlib()
                    .build()?
                    .parse(&fs::read_to_string("html/admin.html").unwrap())?
                    .render(&liquid::object!({}))?
            )
        )
    }().map_err(|err| ErrorInternalServerError(err))
}

fn html_special_chars_reverce(text: &str) -> String {
    text
        .replace("&amp;", "&")
        .replace("&#039;", "'")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

#[derive(Deserialize)]
pub(super) struct Sql {
    sql: String,
}
pub(super) async fn execute_sql(req: HttpRequest, info: web::Json<Sql>) -> Result<String, actix_web::Error> {
    // パスワード取得
    let password =  req.cookie("admin_password")
        .ok_or(ErrorForbidden("パスワードが無効です"))?;
	// データベースに接続
	let conn = common::open_database()?;
    // パスワード確認
    check_server_password(&conn, password.value())?;
    // 処理開始
    conn.execute(&info.sql, [])
        .map_err(|err| ErrorBadRequest(err))?;
    Ok(format!("成功: {}", info.sql))
}

// プレイヤー
#[derive(Serialize)]
pub(super) struct Character {
    eno: i16,
    name: String,
    location: String,
    kins: i32,
}
pub(super) async fn get_characters(req: HttpRequest) -> Result<web::Json<Vec<Character>>, actix_web::Error> {
    // パスワード取得
    let password =  req.cookie("admin_password")
        .ok_or(ErrorForbidden("パスワードが無効です"))?;
    // データベースに接続
	let conn = common::open_database()?;
    // パスワード確認
    check_server_password(&conn, password.value())?;
    // 処理開始
    let mut stmt = conn.prepare("SELECT eno,name,location,kins FROM character")
        .map_err(|err| ErrorInternalServerError(err))?;
    let result = stmt.query_map([], |row| {
        Ok(Character{
            eno: row.get(0)?,
            name: row.get(1)?,
            location: row.get(2)?,
            kins: row.get(3)?,
        })
    })
        .map_err(|err| ErrorInternalServerError(err))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| ErrorInternalServerError(err))?;
    Ok(web::Json(result))
}
#[derive(Deserialize)]
pub(super) struct CharacterData {
    eno: i16,
    location: String,
    kins: i32,
}
pub(super) async fn update_character(req: HttpRequest, info: web::Json<CharacterData>) -> Result<String, actix_web::Error> {
    // パスワード取得
    let password =  req.cookie("admin_password")
        .ok_or(ErrorForbidden("パスワードが無効です"))?;
	// データベースに接続
	let conn = common::open_database()?;
    // パスワード確認
    check_server_password(&conn, password.value())?;
    // 処理開始
    conn.execute("UPDATE character SET location=?2,kins=?3 WHERE eno=?1", params![info.eno, info.location, info.kins]).map_err(|err| ErrorBadRequest(err))?;
    Ok(format!("Eno.{}のロケーションを変更しました", info.eno))
}

// ベースフラグメント
#[derive(Serialize, Deserialize)]
pub(super) struct Fragment {
    id: Option<i32>,
    category: String,
    name: String,
    lore: String,
    status: battle::Status,
    skill: Option<i32>,
}
pub(super) async fn get_fragments(req: HttpRequest) -> Result<web::Json<Vec<Fragment>>, actix_web::Error> {
    // パスワード取得
    let password =  req.cookie("admin_password")
        .ok_or(ErrorForbidden("パスワードが無効です"))?;
    // データベースに接続
	let conn = common::open_database()?;
    // パスワード確認
    check_server_password(&conn, password.value())?;
    // 処理開始
    let mut stmt = conn.prepare("SELECT id,category,name,lore,status,skill FROM base_fragment")
        .map_err(|err| ErrorInternalServerError(err))?;
    let result = stmt.query_map([], |row| {
        Ok(Fragment{
            id: row.get(0)?,
            category: row.get(1)?,
            name: row.get(2)?,
            lore: row.get::<_, String>(3)?.replace("<br>", "\n"),
            status: row.get::<_, [u8; 8]>(4)?.into(),
            skill: row.get(5)?,
        })
    })
        .map_err(|err| ErrorInternalServerError(err))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| ErrorInternalServerError(err))?;
    Ok(web::Json(result))
}
pub(super) async fn update_fragment(req: HttpRequest, info: web::Json<Fragment>) -> Result<String, actix_web::Error> {
    // パスワード取得
    let password =  req.cookie("admin_password")
        .ok_or(ErrorForbidden("パスワードが無効です"))?;
    // データベースに接続
	let conn = common::open_database()?;
    // パスワード確認
    check_server_password(&conn, password.value())?;
    // 処理開始
    let re = Regex::new("\r|\n|\r\n").map_err(|err| ErrorInternalServerError(err))?;
    let lore = html_special_chars_reverce(&re.replace_all(&info.lore, "<br>").to_string());
    let status: [u8; 8] = info.status.into();
    match info.id {
        Some(id) => {
            conn.execute("UPDATE base_fragment SET category=?1,name=?2,lore=?3,status=?4,skill=?5 WHERE id=?6", params![info.category, info.name, lore, status, info.skill, id])
                .map_err(|err| ErrorInternalServerError(err))?;
            Ok(format!("フラグメント編集 {}: {}", id, info.name))
        }
        None => {
            conn.execute("INSERT INTO base_fragment(category,name,lore,status,skill) VALUES(?1,?2,?3,?4,?5)", params![info.category, info.name, lore, status, info.skill])
                .map_err(|err| ErrorInternalServerError(err))?;
            Ok(format!("フラグメント追加 {}: {}", conn.last_insert_rowid(), info.name))
        }
    }
}

// スキル
#[derive(Serialize)]
pub(super) struct Skill {
    id: i32,
    name: String,
    lore: String,
    timing: i8,
    effect: Vec<battle::Command>,
}
pub(super) async fn get_skills(req: HttpRequest) -> Result<web::Json<Vec<Skill>>, actix_web::Error> {
    // パスワード取得
    let password =  req.cookie("admin_password")
        .ok_or(ErrorForbidden("パスワードが無効です"))?;
	// データベースに接続
	let conn = common::open_database()?;
    // パスワード確認
    check_server_password(&conn, password.value())?;
    // 処理開始
    let mut stmt = conn.prepare("SELECT id,name,lore,type,effect FROM skill")
        .map_err(|err| ErrorInternalServerError(err))?;
    let result = stmt.query_map([], |row| {
        Ok(Skill{
            id: row.get(0)?,
            name: row.get(1)?,
            lore: row.get::<_, String>(2)?.replace("<br>", "\n"),
            timing: row.get(3)?,
            effect: battle::Command::convert(row.get(4)?).map_err(|_| rusqlite::Error::InvalidQuery)?,
        })
    })
        .map_err(|err| ErrorInternalServerError(err))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| ErrorInternalServerError(err))?;
    Ok(web::Json(result))
}
#[derive(Deserialize)]
pub(super) struct SkillData {
    id: Option<i32>,
    name: String,
    lore: String,
    timing: i8,
    effect: String,
}
pub(super) async fn update_skill(req: HttpRequest, info: web::Json<SkillData>) -> Result<String, actix_web::Error> {
    // パスワード取得
    let password =  req.cookie("admin_password")
        .ok_or(ErrorForbidden("パスワードが無効です"))?;
	// データベースに接続
	let conn = common::open_database()?;
    // パスワード確認
    check_server_password(&conn, password.value())?;
    // 処理開始
    let re = Regex::new("\r|\n|\r\n").map_err(|err| ErrorInternalServerError(err))?;
    let lore = html_special_chars_reverce(&re.replace_all(&info.lore, "<br>").to_string());
    let command = if battle::Timing::from(info.timing) == battle::Timing::World {
        let id = info.effect.parse::<i16>()
            .map_err(|_| ErrorBadRequest("世界観スキルの効果にはIDを指定する"))?;
        battle::WorldEffect::try_from(id).map_err(|err| ErrorBadRequest(err))?;
        Vec::from([(id >> 8) as u8, id as u8])
    } else {
        let mut command: Vec<u8> = Vec::new();
        for s in info.effect.split(&[' ', ',']) {
            let c = battle::Command::try_from(s.to_string()).map_err(|err| ErrorBadRequest(err))?;
            let c = i16::from(c);
            command.push((c >> 8) as u8);
            command.push(c as u8);
        }
        command
    };
    match info.id {
        Some(id) => {
            conn.execute("UPDATE skill SET name=?1,lore=?2,type=?3,effect=?4 WHERE id=?5", params![info.name, lore, info.timing, command, id])
                .map_err(|err| ErrorInternalServerError(err))?;
            Ok(format!("スキル編集 {}: {}", id, info.name))
        }
        None => {
            conn.execute("INSERT INTO skill(name,lore,type,effect) VALUES(?1,?2,?3,?4)", params![info.name, lore, info.timing, command])
                .map_err(|err| ErrorInternalServerError(err))?;
            Ok(format!("スキル追加 {}: {}", conn.last_insert_rowid(), info.name))
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct PlayersFragment {
    eno: i16,
    slot: Option<i8>,
    category: String,
    name: String,
    lore: String,
    status: battle::Status,
    skill: Option<i32>,
    user: bool,
}
#[derive(Deserialize)]
pub(super) struct PlayerRange {
    start: i16,
    end: i16,
}
pub(super) async fn get_players_fragments(req: HttpRequest, info: web::Query<PlayerRange>) -> Result<web::Json<Vec<PlayersFragment>>, actix_web::Error> {
    // パスワード取得
    let password =  req.cookie("admin_password")
        .ok_or(ErrorForbidden("パスワードが無効です"))?;
    // データベースに接続
	let conn = common::open_database()?;
    // パスワード確認
    check_server_password(&conn, password.value())?;
    // 処理開始
    let mut stmt = conn.prepare("SELECT eno,slot,category,name,lore,status,skill,user FROM fragment WHERE eno>=?1 AND eno<?2 ORDER BY eno ASC,slot ASC")
        .map_err(|err| ErrorInternalServerError(err))?;
    let result = stmt.query_map([info.start, info.end], |row| {
        Ok(PlayersFragment {
            eno: row.get(0)?,
            slot: row.get(1)?,
            category: row.get(2)?,
            name: row.get(3)?,
            lore: row.get(4)?,
            status: row.get::<_, [u8; 8]>(5)?.into(),
            skill: row.get(6)?,
            user: row.get(7)?,
        })
    }).map_err(|err| ErrorInternalServerError(err))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| ErrorInternalServerError(err))?;
    Ok(web::Json(result))
}

#[derive(Deserialize)]
pub(super) struct UpdatePlayersFragment {
    delete: bool,
    eno: i16,
    slot: i8,
    category: String,
    status: battle::Status,
    skill: Option<i32>,
    user: bool,
}
pub(super) async fn update_players_fragment(req: HttpRequest, info: web::Json<UpdatePlayersFragment>) -> Result<String, actix_web::Error> {
    // パスワード取得
    let password =  req.cookie("admin_password")
        .ok_or(ErrorForbidden("パスワードが無効です"))?;
    // データベースに接続
	let conn = common::open_database()?;
    // パスワード確認
    check_server_password(&conn, password.value())?;
    // 処理開始
    if info.delete {
        conn.execute("DELETE FROM fragment WHERE eno=?1 AND slot=?2", params![info.eno, info.slot])
            .map_err(|err| ErrorInternalServerError(err))?;
        Ok(format!("プレイヤーフラグメント削除 Eno.{}({})", info.eno, info.slot))
    } else {
        let status: [u8; 8] = info.status.into();
        conn.execute(
            "UPDATE fragment SET category=?1,status=?2,skill=?3,user=?4 WHERE eno=?5 AND slot=?6",
            params![info.category, status, info.skill, info.user, info.eno, info.slot],
        ).map_err(|err| ErrorInternalServerError(err))?;
        Ok(format!("プレイヤーフラグメント編集 Eno.{}({})", info.eno, info.slot))
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct NPC {
    id: Option<i32>,
    name: String,
    acronym: String,
    color: [u8; 3],
    word: NPCWord,
    status: battle::Status,
    skill: Vec<(i32, Option<String>, Option<String>)>,
    reward: Vec<Fragment>,
}
pub(super) async fn get_npcs(req: HttpRequest) -> Result<web::Json<Vec<NPC>>, actix_web::Error> {
    // パスワード取得
    let password =  req.cookie("admin_password")
        .ok_or(ErrorForbidden("パスワードが無効です"))?;
    // データベースに接続
	let conn = common::open_database()?;
    // パスワード確認
    check_server_password(&conn, password.value())?;
    // 処理開始
    let mut stmt = conn.prepare("SELECT id,name,acronym,color,word,status FROM npc")
        .map_err(|err| ErrorInternalServerError(err))?;
    let result = stmt.query_map([], |row| {
        let id: i32 = row.get(0)?;
        let mut stmt = conn.prepare("SELECT skill,name,word FROM npc_skill WHERE id=?1 ORDER BY slot ASC")?;
        let skill: Vec<(i32, Option<String>, Option<String>)> = stmt.query_map(params![id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
            ))
        })?.collect::<Result<_, _>>()?;
        let mut stmt = conn.prepare("SELECT weight,category,name,lore,status,skill FROM reward WHERE npc=?1")?;
        let reward: Vec<Fragment> = stmt.query_map(params![id], |row| {
            Ok(Fragment{
                id: row.get(0)?,
                category: row.get(1)?,
                name: row.get(2)?,
                lore: row.get::<_, String>(3)?.replace("<br>", "\n"),
                status: row.get::<_, [u8; 8]>(4)?.into(),
                skill: row.get(5)?,
            })
        })?.collect::<Result<_, _>>()?;
        Ok(NPC {
            id: Some(id),
            name: row.get(1)?,
            acronym: row.get(2)?,
            color: row.get(3)?,
            word: serde_json::from_str(&row.get::<_, String>(4)?).map_err(|_| rusqlite::Error::InvalidColumnType(4, "npc.word".to_string(), rusqlite::types::Type::Text))?,
            status: row.get::<_, [u8; 8]>(5)?.into(),
            skill,
            reward,
        })
    }).map_err(|err| ErrorInternalServerError(err))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| ErrorInternalServerError(err))?;
    Ok(web::Json(result))
}
pub(super) async fn update_npc(req: HttpRequest, info: web::Json<NPC>) -> Result<String, actix_web::Error> {
    // パスワード取得
    let password =  req.cookie("admin_password")
        .ok_or(ErrorForbidden("パスワードが無効です"))?;
    // データベースに接続
	let conn = common::open_database()?;
    // パスワード確認
    check_server_password(&conn, password.value())?;
    // 処理開始
    let re = Regex::new("\r|\n|\r\n").map_err(|err| ErrorInternalServerError(err))?;
    let word = serde_json::to_string(&info.word).map_err(|err| ErrorBadRequest(err))?;
    let status: [u8; 8] = info.status.into();
    if let Some(id) = info.id {
        // 更新
        // スキル以外の情報を更新
        conn.execute("UPDATE npc SET name=?2,acronym=?3,color=?4,word=?5,status=?6 WHERE id=?1", params![
            id, info.name, info.acronym, info.color, word, status
        ]).map_err(|err| ErrorInternalServerError(err))?;
        // 一旦スキルを全部削除
        conn.execute("DELETE FROM npc_skill WHERE id=?1", params![id])
            .map_err(|err| ErrorInternalServerError(err))?;
        // 追加しなおし
        let mut i = 0;
        for v in &info.skill {
            i += 1;
            conn.execute("INSERT INTO npc_skill VALUES(?1,?2,?3,?4,?5)", params![id, i, v.0, v.1, v.2])
                .map_err(|err| ErrorInternalServerError(err))?;
        }
        // 報酬も同じく削除
        conn.execute("DELETE FROM reward WHERE npc=?1", params![id])
            .map_err(|err| ErrorInternalServerError(err))?;
        // 追加しなおし
        for v in &info.reward {
            let lore = html_special_chars_reverce(&re.replace_all(&v.lore, "<br>"));
            let status: [u8; 8] = v.status.into();
            conn.execute("INSERT INTO reward VALUES(?1,?2,?3,?4,?5,?6,?7)", params![id, v.id, v.category, v.name, lore, status, v.skill])
                .map_err(|err| ErrorInternalServerError(err))?;
        }
        Ok(format!("NPC更新 ID{}", id))
    } else {
        // 追加
        conn.execute("INSERT INTO npc(name,acronym,color,word,status) VALUES(?1,?2,?3,?4,?5)", params![info.name, info.acronym, info.color, word, status])
            .map_err(|err| ErrorInternalServerError(err))?;
        let id = conn.last_insert_rowid();
        let mut i = 0;
        for v in &info.skill {
            i += 1;
            conn.execute("INSERT INTO npc_skill VALUES(?1,?2,?3,?4,?5)", params![id, i, v.0, v.1, v.2])
                .map_err(|err| ErrorInternalServerError(err))?;
        }
        for v in &info.reward {
            let lore = html_special_chars_reverce(&re.replace_all(&v.lore, "<br>"));
            let status: [u8; 8] = v.status.into();
            conn.execute("INSERT INTO reward VALUES(?1,?2,?3,?4,?5,?6,?7)", params![id, v.id, v.category, v.name, lore, status, v.skill])
                .map_err(|err| ErrorInternalServerError(err))?;
        }
        Ok(format!("NPC追加 ID{}", id))
    }
}

pub(super) async fn add_players_fragment(req: HttpRequest, info: web::Json<PlayersFragment>) -> Result<String, actix_web::Error> {
    // パスワード取得
    let password =  req.cookie("admin_password")
        .ok_or(ErrorForbidden("パスワードが無効です"))?;
    // データベースに接続
	let conn = common::open_database()?;
    // パスワード確認
    check_server_password(&conn, password.value())?;
    // 処理開始
    let re = Regex::new("\r|\n|\r\n").map_err(|err| ErrorInternalServerError(err))?;
    let lore = html_special_chars_reverce(&re.replace_all(&info.lore, "<br>"));
    let status: [u8; 8] = info.status.into();
    if let Some(slot) = info.slot {
        conn.execute(
            "UPDATE fragment SET category=?3,name=?4,lore=?5,status=?6,skill=?7 WHERE eno=?1 AND slot=?2",
                params![info.eno, slot, info.category, info.name, lore, status, info.skill],
            ).map_err(|err| ErrorInternalServerError(err))?;
            Ok(format!("フラグメント編集 Eno.{}", info.eno))
    } else {
        match common::get_empty_slot(&conn, info.eno).map_err(|err| ErrorInternalServerError(err))? {
            Some(slot) => {
                conn.execute(
                    "INSERT INTO fragment(eno,slot,category,name,lore,status,skill) VALUES(?1,?2,?3,?4,?5,?6,?7)",
                    params![info.eno, slot, info.category, info.name, lore, status, info.skill],
                ).map_err(|err| ErrorInternalServerError(err))?;
                Ok(format!("フラグメント追加 Eno.{}", info.eno))
            }
            None => {
                Err(ErrorBadRequest("対象のスロットに空きがありません"))
            }
        }
    }
}