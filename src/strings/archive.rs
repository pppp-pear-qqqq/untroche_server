use std::hash::{DefaultHasher, Hash, Hasher};

use actix_web::{error::{ErrorBadRequest, ErrorInternalServerError}, web::{self, Json}};
use rusqlite::{named_params, params};
use serde::{Deserialize, Serialize};

use super::{battle, common};

#[derive(Deserialize)]
pub(super) struct Player {
	eno: i16,
	password: String,
}
#[derive(Serialize)]
pub(super) struct Skill {
	name: Option<String>,
	word: Option<String>,
	default_name: String,
	lore: String,
	timing: battle::Timing,
	effect: battle::Effect,
}
#[derive(Serialize)]
pub(super) struct Fragment {
	slot: i8,
	category: String,
	name: String,
	lore: String,
	status: battle::Status,
	skill: Option<Skill>,
}
pub(super) async fn get_fragments(info: web::Query<Player>) -> Result<Json<Vec<Fragment>>, actix_web::Error> {
	// パスワードのハッシュ化
	let mut hasher = DefaultHasher::new();
    info.password.hash(&mut hasher);
    // データベースに接続
    let conn = common::open_database()?;
    // Enoを取得
    let eno = conn.query_row(
        "SELECT eno FROM user WHERE eno=?1 AND password=?2",
        params![info.eno, hasher.finish() as i64],
        |row| row.get::<usize, i16>(0),
    ).map_err(|err| match err {
        rusqlite::Error::QueryReturnedNoRows => ErrorBadRequest("Enoまたはパスワードが違います"),
        _ => ErrorBadRequest(err)
    })?;
	// フラグメント取得
	let mut stmt = conn.prepare("WITH f AS (SELECT slot,category,name,lore,status,skill,skillname,skillword FROM fragment WHERE eno=?1) SELECT f.slot,f.category,f.name,f.lore,f.status,f.skillname,f.skillword,s.name,s.lore,s.type,s.effect FROM f LEFT OUTER JOIN skill s ON f.skill=s.id")
		.map_err(|err| ErrorInternalServerError(err))?;
	let result = stmt.query_map(params![eno], |row| {
		let skill = if let (Some(default_name), Some(lore), Some(timing), Some(effect)) = (row.get(7)?, row.get(8)?, row.get::<_, Option<i8>>(9)?, row.get(10)?) {
			let timing = timing.into();
			let effect = match timing {
				battle::Timing::World => battle::Effect::World(
					battle::WorldEffect::convert(effect)
						.map_err(|_| rusqlite::Error::InvalidColumnType(10, "skill.effect".into(), rusqlite::types::Type::Blob))?
				),
				_ => battle::Effect::Formula(
					battle::Command::convert(effect)
						.map_err(|_| rusqlite::Error::InvalidColumnType(10, "skill.effect".into(), rusqlite::types::Type::Blob))?
				),
			};
			Some(Skill{
				name: row.get(5)?,
				word: row.get(6)?,
				default_name,
				lore,
				timing,
				effect,
			})
		} else {
			None
		};
		Ok(Fragment{
			slot: row.get(0)?,
			category: row.get(1)?,
			name: row.get(2)?,
			lore: row.get(3)?,
			status: battle::Status::from(row.get::<_, [u8; 8]>(4)?),
			skill,
		})
	}).map_err(|err| ErrorInternalServerError(err))?
		.collect::<Result<_, _>>()
		.map_err(|err| ErrorInternalServerError(err))?;
	Ok(Json(result))
}

#[derive(Deserialize)]
pub(super) struct SearchOrder {
	location: String,
	from: String,
	to: String,
	word: Option<String>,
}
#[derive(Serialize)]
pub(super) struct Chat {
	id: i32,
	timestamp: String,
	from: i16,
	to: Option<i16>,
	location: Option<String>,
	acronym: String,
	color: [u8; 3],
	name: String,
	word: String,
}
pub(super) async fn get_timeline(info: web::Query<SearchOrder>) -> Result<Json<Vec<Chat>>, actix_web::Error> {
	// 入力値検証
	if info.location.len() >= 128 || info.from.len() >= 128 || info.to.len() >= 128 || info.word.is_some() && info.word.as_ref().unwrap().len() >= 256 {
		return Err(ErrorBadRequest("入力が長すぎます"));
	}
	// パラメータ生成
	let mut sql = Vec::new();
	let mut params = named_params! {}.to_vec();
	if !info.location.is_empty() && info.location != "*" {
		sql.push("location=:location");
		params.push((":location", &info.location));
	}
	let (plus, minus) = if !info.from.is_empty() || !info.to.is_empty() {
		let mut from_plus = Vec::new();
		let mut from_minus = Vec::new();
		let mut to_plus = Vec::new();
		let mut to_minus = Vec::new();
		if !info.from.is_empty() {
			for from in info.from.split(',').collect::<Vec<_>>() {
				let from = from.trim();
				let v = from.parse::<i16>().map_err(|_| ErrorBadRequest("発言者に数値でないものが含まれています"))?;
				if from.chars().next() != Some('-') {
					from_plus.push(v);
				} else {
					from_minus.push(v * -1);
				}
			}
		}
		if !info.to.is_empty() {
			for to in info.to.split(',').collect::<Vec<_>>() {
				let to = to.trim();
				let v = to.parse::<i16>().map_err(|_| ErrorBadRequest("発言者に数値でないものが含まれています"))?;
				if to.chars().next() != Some('-') {
					to_plus.push(v);
				} else {
					to_minus.push(v * -1);
				}
			}
		}
		let mut plus = Vec::new();
		let mut minus = Vec::new();
		if !from_plus.is_empty() {
			plus.push(format!("from_eno IN ({})", from_plus.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")));
		}
		if !to_plus.is_empty() {
			plus.push(format!("to_eno IN ({})", to_plus.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")));
		}
		if !from_minus.is_empty() {
			minus.push(format!("from_eno NOT IN ({})", from_minus.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")));
		}
		if !to_minus.is_empty() {
			minus.push(format!("to_eno NOT IN ({})", to_minus.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")));
		}
		(
			if !plus.is_empty() {
				Some(format!("({})", plus.join(" OR ")))
			} else {
				None
			},
			if !minus.is_empty() {
				Some(format!("({})", minus.join(" OR ")))
			} else {
				None
			},
		)
		// (from_eno IN (...) OR to_eno IN (...)) AND (from_eno NOT IN (...) OR to_eno NOT IN (...))
	} else {
		(None, None)
	};
	if let Some(plus) = &plus {
		sql.push(plus);
	}
	if let Some(minus) = &minus {
		sql.push(minus);
	}
	if info.word.is_some() {
		sql.push("word='%:word%'");
		params.push((":word", &info.word));
	}
    // データベースに接続
    let conn = common::open_database()?;
	// 取得
	let mut stmt = conn.prepare(format!(
		"SELECT id,timestamp,from_eno,to_eno,location,acronym,color,name,word FROM timeline WHERE live=true{} LIMIT 1000",
		if !sql.is_empty() {
			format!(" AND {}", sql.join(" AND "))
		} else {
			String::new()
		}
	).as_str()).map_err(|err| ErrorInternalServerError(err))?;
	let result = stmt.query_map(params.as_slice(), |row| {
		Ok(Chat{
			id: row.get(0)?,
			timestamp: row.get(1)?,
			from: row.get(2)?,
			to: row.get(3)?,
			location: row.get(4)?,
			acronym: row.get(5)?,
			color: row.get(6)?,
			name: row.get(7)?,
			word: row.get(8)?,
		})
	}).map_err(|err| ErrorInternalServerError(err))?
		.collect::<Result<_, _>>()
		.map_err(|err| ErrorInternalServerError(err))?;
	Ok(Json(result))
}