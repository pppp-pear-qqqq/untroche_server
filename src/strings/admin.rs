use actix_web::{web, HttpResponse, error::{ErrorBadRequest, ErrorForbidden, ErrorInternalServerError}};
use rusqlite::{params, Connection};
use serde::Deserialize;
use super::{
    battle::{self, Timing},
    common,
};

// データベースに保存されているパスワードを取得
fn check_server_password(conn: &Connection, password: String) -> Result<bool, rusqlite::Error> {
    Ok(password == conn.query_row("SELECT password FROM server", [], |row| {
        row.get::<usize, String>(0)
    })?)
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

fn create_skill(
    conn: &Connection,
    name: &str,
    lore: &str,
    timing: battle::Timing,
    commands: Vec<battle::Command>,
) -> Result<i64, rusqlite::Error> {
    let mut buf = Vec::<u8>::new();
    for command in commands {
        let v = command.to_i16();
        buf.push((v >> 8) as u8);
        buf.push(v as u8);
    }
    conn.execute(
        "INSERT INTO skill(name,lore,type,effect) VALUES(?1,?2,?3,?4)",
        params![name, lore, timing.to_i8(), buf],
    )?;
    Ok(conn.last_insert_rowid())
}

// 管理者用ページ
pub async fn index(path: web::Path<String>) -> Result<HttpResponse, actix_web::Error> {
	// データベースに接続
	let conn = common::open_database()?;
    // URLに含まれるパスワード部分を取得して確認
    if check_server_password(&conn, path.into_inner())
		.map_err(|err| ErrorInternalServerError(err))?
	{
        // パスワードが一致していればいい感じのを返す
        Ok(HttpResponse::Ok().body(
            common::liquid_build("html/admin.html")
                .map_err(|err| ErrorInternalServerError(err))?
                .render(&liquid::object!({}))
                .map_err(|err| ErrorInternalServerError(err))?,
        ))
    } else {
        // 一致していなければエラー
        Err(ErrorForbidden("パスワードが違います"))
    }
}

#[derive(Deserialize)]
pub struct MakeData {
    value: String,
}
pub async fn make_skill(path: web::Path<String>, info: web::Json<MakeData>) -> Result<String, actix_web::Error> {
	// データベースに接続
	let conn = common::open_database()?;
    // パスワード確認
    if check_server_password(&conn, path.into_inner())
		.map_err(|err| ErrorInternalServerError(err))?
	{
        // スキルID スキル名 説明文 タイミング コマンド...
        let mut sp = info.value.split('\t');
        || -> Option<String> {
            let name = sp.next()?;
            let lore = sp.next()?;
            let timing = match sp.next()? {
                "通常" => Timing::Active,
                "反応" => Timing::Reactive,
                "開始" => Timing::Start,
                "勝利" => Timing::Win,
                "敗北" => Timing::Lose,
                "無感" => Timing::None,
                _ => Timing::None,
            };
            let mut command = Vec::new();
            for s in sp.next()?.split(',') {
                match s {
                    "HP" => command.push(battle::Command::Uhp),
                    "MP" => command.push(battle::Command::Ump),
                    "ATK" => command.push(battle::Command::Uatk),
                    "TEC" => command.push(battle::Command::Utec),
                    "自身HP" => command.push(battle::Command::Uhp),
                    "自身MP" => command.push(battle::Command::Ump),
                    "自身ATK" => command.push(battle::Command::Uatk),
                    "自身TEC" => command.push(battle::Command::Utec),
                    "相手HP" => command.push(battle::Command::Thp),
                    "相手MP" => command.push(battle::Command::Tmp),
                    "相手ATK" => command.push(battle::Command::Tatk),
                    "相手TEC" => command.push(battle::Command::Ttec),
                    "間合値" => command.push(battle::Command::ValueRange),
                    "逃走値" => command.push(battle::Command::ValueEscape),

                    "正" => command.push(battle::Command::Plus),
                    "負" => command.push(battle::Command::Minus),
                    "+" => command.push(battle::Command::Add),
                    "-" => command.push(battle::Command::Sub),
                    "*" => command.push(battle::Command::Mul),
                    "/" => command.push(battle::Command::Div),
                    "%" => command.push(battle::Command::Mod),
                    "~" => command.push(battle::Command::RandomRange),

                    "消耗" => command.push(battle::Command::Cost),
                    "強命消耗" => command.push(battle::Command::ForceCost),
                    "間合" => command.push(battle::Command::Range),
                    "確率" => command.push(battle::Command::Random),
                    "中断" => command.push(battle::Command::Break),

                    "攻撃" => command.push(battle::Command::Attack),
                    "貫通攻撃" => command.push(battle::Command::ForceAttack),
                    "精神攻撃" => command.push(battle::Command::MindAttack),
                    "回復" => command.push(battle::Command::Heal),
                    "自傷" => command.push(battle::Command::SelfDamage),
                    "集中" => command.push(battle::Command::Concentrate),
                    "ATK強化" => command.push(battle::Command::BuffAtk),
                    "TEC強化" => command.push(battle::Command::BuffTec),
                    "移動" => command.push(battle::Command::Move),
                    "間合変更" => command.push(battle::Command::ChangeRange),
                    "逃走ライン" => command.push(battle::Command::ChangeEscapeRange),
                    "対象変更" => command.push(battle::Command::ChangeUser),

                    other => {command.push(battle::Command::Value(other.parse::<i16>().ok()?))},
                }
            }
            match create_skill(&conn, name, lore, timing, command) {
                Ok(id) => {
                    Some(format!("スキルの作成完了 ID{} : {}", id, name))
                },
                Err(err) => {
                    Some(format!("{}", err))
                }
            }
        }().ok_or(ErrorBadRequest("スキルの情報が不十分です"))
    } else {
        // 一致していなければエラー
        Err(ErrorForbidden("パスワードが違います"))
    }
}

pub async fn make_fragment(path: web::Path<String>, info: web::Json<MakeData>) -> Result<String, actix_web::Error> {
    // データベースに接続
	let conn = common::open_database()?;
    if check_server_password(&conn, path.into_inner())
		.map_err(|err| ErrorInternalServerError(err))?
	{
        let mut sp = info.value.split('\t');
        let err = "フラグメントの情報が不完全です";
        let category = sp.next().ok_or(ErrorBadRequest(err))?;
        let name = sp.next().ok_or(ErrorBadRequest(err))?;
        let lore = sp.next().ok_or(ErrorBadRequest(err))?;
        let hp = sp.next().ok_or(ErrorBadRequest(err))?.parse::<i16>().map_err(|_| ErrorBadRequest(err))?;
        let mp = sp.next().ok_or(ErrorBadRequest(err))?.parse::<i16>().map_err(|_| ErrorBadRequest(err))?;
        let atk = sp.next().ok_or(ErrorBadRequest(err))?.parse::<i16>().map_err(|_| ErrorBadRequest(err))?;
        let tec = sp.next().ok_or(ErrorBadRequest(err))?.parse::<i16>().map_err(|_| ErrorBadRequest(err))?;
        let skill = sp.next().ok_or(ErrorBadRequest(err))?;
        let status = [
            (hp >> 8) as u8, hp as u8,
            (mp >> 8) as u8, mp as u8,
            (atk >> 8) as u8, atk as u8,
            (tec >> 8) as u8, tec as u8,
        ];
        if skill == "なし" {
            conn.execute("INSERT INTO base_fragment(category,name,lore,status) VALUES(?1,?2,?3,?4)", params![category, name, lore, status])
                .map_err(|err| ErrorInternalServerError(err))?;
        } else {
            conn.execute("INSERT INTO base_fragment(category,name,lore,status,skill) VALUES(?1,?2,?3,?4,(SELECT id FROM skill WHERE name=?5 LIMIT 1))", params![category, name, lore, status, skill])
                .map_err(|err| ErrorInternalServerError(err))?;
        }
        Ok(format!("フラグメント作成完了 ID{} : {}", conn.last_insert_rowid(), name))
    } else {
        Err(ErrorForbidden("パスワードが違います"))
    }
}