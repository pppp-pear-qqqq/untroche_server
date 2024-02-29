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
fn check_server_password(conn: &Connection, password: &str) -> Result<(), actix_web::Error> {
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

// サーバー起動時に実行する関数
#[allow(dead_code)]
pub fn preset() -> Result<(), rusqlite::Error> {
    // let conn = Connection::open(common::DATABASE)?;
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

#[derive(Serialize)]
struct Fragment {
    id: i32,
    category: String,
    name: String,
    lore: String,
    hp: i16,
    mp: i16,
    atk: i16,
    tec: i16,
    skill: Option<i32>,
}
#[derive(Serialize)]
struct Skill {
    id: i32,
    name: String,
    lore: String,
    timing: i8,
    effect: String,
}
#[derive(Serialize)]
struct Character {
    eno: i16,
    name: String,
    location: String,
}
#[derive(Deserialize)]
pub(super) struct Password {
    pass: String,
}
// 管理者用ページ
pub async fn index(pass: web::Query<Password>) -> Result<HttpResponse, actix_web::Error> {
	// データベースに接続
	let conn = common::open_database()?;
    // URLに含まれるパスワード部分を取得して確認
    check_server_password(&conn, &pass.pass)?;
    // パスワードが一致していればいい感じのを返す
    // フラグメント
    let mut stmt = conn.prepare("SELECT id,category,name,lore,status,skill FROM base_fragment")
        .map_err(|err| ErrorInternalServerError(err))?;
    let fragments = stmt.query_map([], |row| {
        let status: [u8; 8] = row.get(4)?;
        Ok(Fragment{
            id: row.get(0)?,
            category: row.get(1)?,
            name: row.get(2)?,
            lore: row.get::<_, String>(3)?.replace("<br>", "\n"),
            hp: (status[0] as i16) << 8 | status[1] as i16,
            mp: (status[2] as i16) << 8 | status[3] as i16,
            atk: (status[4] as i16) << 8 | status[5] as i16,
            tec: (status[6] as i16) << 8 | status[7] as i16,
            skill: row.get(5)?,
        })
    })
        .map_err(|err| ErrorInternalServerError(err))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| ErrorInternalServerError(err))?;
    // スキル
    let mut stmt = conn.prepare("SELECT id,name,lore,type,effect FROM skill")
        .map_err(|err| ErrorInternalServerError(err))?;
    let skills = stmt.query_map([], |row| {
        let effect = battle::Command::convert(row.get(4)?).map_err(|_| rusqlite::Error::InvalidQuery)?;
        let effect = effect.iter().map(|x| x.to_string()).collect::<Vec<String>>();
        Ok(Skill{
            id: row.get(0)?,
            name: row.get(1)?,
            lore: row.get::<_, String>(2)?.replace("<br>", "\n"),
            timing: row.get(3)?,
            effect: effect.join(" "),
        })
    })
        .map_err(|err| ErrorInternalServerError(err))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| ErrorInternalServerError(err))?;
    // キャラクター
    let mut stmt = conn.prepare("SELECT eno,name,location FROM character")
        .map_err(|err| ErrorInternalServerError(err))?;
    let characters = stmt.query_map([], |row| {
        Ok(Character{
            eno: row.get(0)?,
            name: row.get(1)?,
            location: row.get(2)?,
        })
    })
        .map_err(|err| ErrorInternalServerError(err))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| ErrorInternalServerError(err))?;
    || -> Result<HttpResponse, liquid::Error> {
        Ok(HttpResponse::Ok()
            .cookie(Cookie::build("admin_password", &pass.pass)
                .finish()
            ).body(
                liquid::ParserBuilder::with_stdlib()
                    .build()?
                    .parse(&fs::read_to_string("html/admin.html").unwrap())?
                    .render(&liquid::object!({"fragment":fragments, "skill":skills, "character":characters}))?
            )
        )
    }().map_err(|err| ErrorInternalServerError(err))
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

#[derive(Deserialize)]
pub(super) struct CharacterData {
    eno: i16,
    location: String,
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
    conn.execute("UPDATE character SET location=?1 WHERE eno=?2", params![info.location, info.eno]).map_err(|err| ErrorBadRequest(err))?;
    Ok(format!("Eno.{}のロケーションを変更しました", info.eno))
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
    let lore = common::html_special_chars(info.lore.clone());
    let lore = re.replace_all(&lore, "<br>");
    let mut command: Vec<u8> = Vec::new();
    for s in info.effect.split(&[' ', ',']) {
        let c = match s {
            "HP" => battle::Command::Uhp,
            "MP" => battle::Command::Ump,
            "ATK" => battle::Command::Uatk,
            "TEC" => battle::Command::Utec,
            "自身HP" => battle::Command::Uhp,
            "自身MP" => battle::Command::Ump,
            "自身ATK" => battle::Command::Uatk,
            "自身TEC" => battle::Command::Utec,
            "相手HP" => battle::Command::Thp,
            "相手MP" => battle::Command::Tmp,
            "相手ATK" => battle::Command::Tatk,
            "相手TEC" => battle::Command::Ttec,
            "間合値" => battle::Command::ValueRange,
            "逃走値" => battle::Command::ValueEscape,

            "正" => battle::Command::Plus,
            "負" => battle::Command::Minus,
            "+" => battle::Command::Add,
            "-" => battle::Command::Sub,
            "*" => battle::Command::Mul,
            "/" => battle::Command::Div,
            "%" => battle::Command::Mod,
            "~" => battle::Command::RandomRange,

            "消耗" => battle::Command::Cost,
            "強命消耗" => battle::Command::ForceCost,
            "間合" => battle::Command::Range,
            "確率" => battle::Command::Random,
            "中断" => battle::Command::Break,

            "攻撃" => battle::Command::Attack,
            "貫通攻撃" => battle::Command::ForceAttack,
            "精神攻撃" => battle::Command::MindAttack,
            "回復" => battle::Command::Heal,
            "自傷" => battle::Command::SelfDamage,
            "集中" => battle::Command::Concentrate,
            "ATK強化" => battle::Command::BuffAtk,
            "TEC強化" => battle::Command::BuffTec,
            "移動" => battle::Command::Move,
            "間合変更" => battle::Command::ChangeRange,
            "逃走ライン" => battle::Command::ChangeEscapeRange,
            "対象変更" => battle::Command::ChangeUser,

            other => battle::Command::Value(other.parse::<i16>().map_err(|err| ErrorInternalServerError(err))?),
        }.to_i16();
        command.push((c >> 8) as u8);
        command.push(c as u8);
    }
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

#[derive(Deserialize)]
pub(super) struct FragmentData {
    id: Option<i32>,
    category: String,
    name: String,
    lore: String,
    hp: i16,
    mp: i16,
    atk: i16,
    tec: i16,
    skill: Option<i32>,
}
pub(super) async fn update_fragment(req: HttpRequest, info: web::Json<FragmentData>) -> Result<String, actix_web::Error> {
    // パスワード取得
    let password =  req.cookie("admin_password")
        .ok_or(ErrorForbidden("パスワードが無効です"))?;
    // データベースに接続
	let conn = common::open_database()?;
    // パスワード確認
    check_server_password(&conn, password.value())?;
    // 処理開始
    let re = Regex::new("\r|\n|\r\n").map_err(|err| ErrorInternalServerError(err))?;
    let lore = common::html_special_chars(info.lore.clone());
    let lore = re.replace_all(&lore, "<br>");
    let status = [
        (info.hp >> 8) as u8, info.hp as u8,
        (info.mp >> 8) as u8, info.mp as u8,
        (info.atk >> 8) as u8, info.atk as u8,
        (info.tec >> 8) as u8, info.tec as u8,
    ];
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