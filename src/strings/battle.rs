use std::{collections::HashMap, num::ParseIntError, ops};

use rand::distributions::{Distribution, WeightedIndex};
use rusqlite::{params, types::Type, Connection};
use serde::{Deserialize, Serialize};

use super::common;

static SYSTEM: &str = "strings";

#[derive(Clone, PartialEq)]
#[allow(dead_code)]
pub enum Command {
    Value(i16),
    Uhp,
    Ump,
    Uatk,
    Utec,
    Thp,
    Tmp,
    Tatk,
    Ttec,
    ValueRange,
    ValueEscape,

    Plus,
    Minus,

    Add,
    Sub,
    Mul,
    Div,
    Mod,
    RandomRange,

    Cost,
    ForceCost,
    Range,
    Random,
    Break,

    Attack,
    ForceAttack,
    MindAttack,
    Heal,
    SelfDamage,
    Concentrate,
    BuffAtk,
    BuffTec,
    Move,
    ChangeRange,
    ChangeEscapeRange,
    ChangeUser,

    BreakToEnd,
}
impl From<Command> for i16 {
    fn from(value: Command) -> Self {
        match value {
            Command::Value(value) => value,
            
            Command::Uhp => -0x1,
            Command::Ump => -0x2,
            Command::Uatk => -0x3,
            Command::Utec => -0x4,
            Command::Thp => -0x5,
            Command::Tmp => -0x6,
            Command::Tatk => -0x7,
            Command::Ttec => -0x8,
            Command::ValueRange => -0x9,
            Command::ValueEscape => -0xa,

            Command::Plus => -0x11,
            Command::Minus => -0x12,
            Command::Add => -0x13,
            Command::Sub => -0x14,
            Command::Mul => -0x15,
            Command::Div => -0x16,
            Command::Mod => -0x17,
            Command::RandomRange => -0x18,

            Command::Cost => -0x21,
            Command::ForceCost => -0x22,
            Command::Range => -0x23,
            Command::Random => -0x24,
            Command::Break => -0x25,

            Command::Attack => -0x31,
            Command::ForceAttack => -0x32,
            Command::MindAttack => -0x33,
            Command::Heal => -0x34,
            Command::SelfDamage => -0x35,
            Command::Concentrate => -0x36,
            Command::BuffAtk => -0x37,
            Command::BuffTec => -0x38,
            Command::Move => -0x39,
            Command::ChangeRange => -0x3a,
            Command::ChangeEscapeRange => -0x3b,
            Command::ChangeUser => -0x3c,

            Command::BreakToEnd => -0x51,
        }
    }
}
impl From<i16> for Command {
    fn from(value: i16) -> Self {
        match value {
            -0x1 => Self::Uhp,
            -0x2 => Self::Ump,
            -0x3 => Self::Uatk,
            -0x4 => Self::Utec,
            -0x5 => Self::Thp,
            -0x6 => Self::Tmp,
            -0x7 => Self::Tatk,
            -0x8 => Self::Ttec,
            -0x9 => Self::ValueRange,
            -0xa => Self::ValueEscape,

            -0x11 => Self::Plus,
            -0x12 => Self::Minus,
            -0x13 => Self::Add,
            -0x14 => Self::Sub,
            -0x15 => Self::Mul,
            -0x16 => Self::Div,
            -0x17 => Self::Mod,
            -0x18 => Self::RandomRange,

            -0x21 => Self::Cost,
            -0x22 => Self::ForceCost,
            -0x23 => Self::Range,
            -0x24 => Self::Random,
            -0x25 => Self::Break,

            -0x31 => Self::Attack,
            -0x32 => Self::ForceAttack,
            -0x33 => Self::MindAttack,
            -0x34 => Self::Heal,
            -0x35 => Self::SelfDamage,
            -0x36 => Self::Concentrate,
            -0x37 => Self::BuffAtk,
            -0x38 => Self::BuffTec,
            -0x39 => Self::Move,
            -0x3a => Self::ChangeRange,
            -0x3b => Self::ChangeEscapeRange,
            -0x3c => Self::ChangeUser,

            -0x51 => Self::BreakToEnd,

            _ => Command::Value(value),
        }
    }
}
impl From<Command> for String {
    fn from(value: Command) -> Self {
        match value {
            Command::Value(v) => v.to_string(),
            Command::Uhp => String::from("HP"),
            Command::Ump => String::from("MP"),
            Command::Uatk => String::from("ATK"),
            Command::Utec => String::from("TEC"),
            Command::Thp => String::from("相手HP"),
            Command::Tmp => String::from("相手MP"),
            Command::Tatk => String::from("相手ATK"),
            Command::Ttec => String::from("相手TEC"),
            Command::ValueRange => String::from("間合値"),
            Command::ValueEscape => String::from("逃走値"),
            Command::Plus => String::from("正"),
            Command::Minus => String::from("負"),
            Command::Add => String::from("+"),
            Command::Sub => String::from("-"),
            Command::Mul => String::from("*"),
            Command::Div => String::from("/"),
            Command::Mod => String::from("%"),
            Command::RandomRange => String::from("~"),
            Command::Cost => String::from("消耗"),
            Command::ForceCost => String::from("強命消耗"),
            Command::Range => String::from("間合"),
            Command::Random => String::from("確率"),
            Command::Break => String::from("中断"),
            Command::Attack => String::from("攻撃"),
            Command::ForceAttack => String::from("貫通攻撃"),
            Command::MindAttack => String::from("精神攻撃"),
            Command::Heal => String::from("回復"),
            Command::SelfDamage => String::from("自傷"),
            Command::Concentrate => String::from("集中"),
            Command::BuffAtk => String::from("ATK変化"),
            Command::BuffTec => String::from("TEC変化"),
            Command::Move => String::from("移動"),
            Command::ChangeRange => String::from("間合変更"),
            Command::ChangeEscapeRange => String::from("逃走ライン"),
            Command::ChangeUser => String::from("対象変更"),
            Command::BreakToEnd => String::from("中断時終了"),
        }
    }
}
impl TryFrom<String> for Command {
    type Error = ParseIntError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "HP" => Ok(Self::Uhp),
            "MP" => Ok(Self::Ump),
            "ATK" => Ok(Self::Uatk),
            "TEC" => Ok(Self::Utec),
            "相手HP" => Ok(Self::Thp),
            "相手MP" => Ok(Self::Tmp),
            "相手ATK" => Ok(Self::Tatk),
            "相手TEC" => Ok(Self::Ttec),
            "間合値" => Ok(Self::ValueRange),
            "逃走値" => Ok(Self::ValueEscape),
            "正" => Ok(Self::Plus),
            "負" => Ok(Self::Minus),
            "+" => Ok(Self::Add),
            "-" => Ok(Self::Sub),
            "*" => Ok(Self::Mul),
            "/" => Ok(Self::Div),
            "%" => Ok(Self::Mod),
            "~" => Ok(Self::RandomRange),
            "消耗" => Ok(Self::Cost),
            "強命消耗" => Ok(Self::ForceCost),
            "間合" => Ok(Self::Range),
            "確率" => Ok(Self::Random),
            "中断" => Ok(Self::Break),
            "攻撃" => Ok(Self::Attack),
            "貫通攻撃" => Ok(Self::ForceAttack),
            "精神攻撃" => Ok(Self::MindAttack),
            "回復" => Ok(Self::Heal),
            "自傷" => Ok(Self::SelfDamage),
            "集中" => Ok(Self::Concentrate),
            "ATK変化" => Ok(Self::BuffAtk),
            "TEC変化" => Ok(Self::BuffTec),
            "移動" => Ok(Self::Move),
            "間合変更" => Ok(Self::ChangeRange),
            "逃走ライン" => Ok(Self::ChangeEscapeRange),
            "対象変更" => Ok(Self::ChangeUser),
            "中断時終了" => Ok(Self::BreakToEnd),
            _ => Ok(Self::Value(value.parse::<i16>()?)),
        }
    }
}
impl Command {
    pub fn convert(blob: Vec<u8>) -> Result<Vec<Command>, String> {
        || -> Option<_> {
            let mut skill = Vec::<Command>::new();
            let mut i: usize = 0;
            while i < blob.len() {
                skill.push(Self::from((*blob.get(i)? as i16) << 8 | *blob.get(i + 1)? as i16));
                i += 2;
            }
            Some(skill)
        }().ok_or("blobのサイズが不正です".to_string())
    }
}
impl Serialize for Command {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        if let Self::Value(v) = self {
            serializer.serialize_i16(*v)
        } else {
            serializer.serialize_str(&String::from(self.to_owned()))
        }
    }
}

#[derive(Clone, PartialEq)]
#[allow(dead_code)]
pub enum Timing {
    Active,
    Reactive,
    Start,
    Win,
    Lose,
    Escape,
    World,
    None,
}
impl From<i8> for Timing {
    fn from(v: i8) -> Timing {
        match v {
            0 => Self::Active,
            1 => Self::Reactive,
            2 => Self::Start,
            3 => Self::Win,
            4 => Self::Lose,
            5 => Self::Escape,
            -1 => Self::World,
            _ => Self::None,
        }
    }
}
impl Serialize for Timing {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        serializer.serialize_str(match self {
            Self::Active => "通常",
            Self::Reactive => "反応",
            Self::Start => "開始",
            Self::Win => "勝利",
            Self::Lose => "敗北",
            Self::Escape => "逃走",
            Self::World => "世界観",
            Self::None => "無感",
        })
    }
}

#[derive(Clone)]
struct Skill {
    default_name: String,
    name: Option<String>,
    word: Option<String>,
    timing: Timing,
    formula: Vec<Command>,
}
impl Skill {
    fn new(default_name: String, name: Option<String>, word: Option<String>, timing: Timing, formula: Vec<Command>) -> Skill {
        Skill { default_name, name, word, timing, formula }
    }
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub(super) struct Status {
    hp: i16,
    mp: i16,
    atk: i16,
    tec: i16,
}
impl From<[u8; 8]> for Status {
    fn from(value: [u8; 8]) -> Self {
        Self {
            hp: (value[0] as i16) << 8 | value[1] as i16,
            mp: (value[2] as i16) << 8 | value[3] as i16,
            atk: (value[4] as i16) << 8 | value[5] as i16,
            tec: (value[6] as i16) << 8 | value[7] as i16,
        }
    }
}
impl From<Status> for [u8; 8] {
    fn from(value: Status) -> Self {
        [
            (value.hp >> 8) as u8, value.hp as u8,
            (value.mp >> 8) as u8, value.mp as u8,
            (value.atk >> 8) as u8, value.atk as u8,
            (value.tec >> 8) as u8, value.tec as u8,
        ]
    }
}
impl Status {
    fn new() -> Self {
        Self { hp: 0, mp: 0, atk: 0, tec: 0 }
    }
}
impl ops::AddAssign<Status> for Status {
    fn add_assign(&mut self, rhs: Status) {
        self.hp += rhs.hp;
        self.mp += rhs.mp;
        self.atk += rhs.atk;
        self.tec += rhs.tec;
    }
}
impl ops::Add<Status> for Status {
    type Output = Status;

    fn add(self, rhs: Status) -> Self::Output {
        Self::Output {
            hp: self.hp + rhs.hp,
            mp: self.mp + rhs.mp,
            atk: self.atk + rhs.atk,
            tec: self.tec + rhs.tec,
        }
    }
}

#[derive(Clone)]
struct Character {
    eno: i16,
    name: String,
    acronym: char,
    color: [u8; 3],
    word: HashMap<String, String>,
    status: Status,
    skill: Vec<(Skill, bool)>,
}

#[derive(PartialEq)]
pub(super) enum BattleResult {
    Win(usize),
    Draw,
    Escape,
}
impl BattleResult {
    fn as_str(&self) -> &str {
        match self {
            BattleResult::Win(i) => get_side(&i),
            BattleResult::Draw => "draw",
            BattleResult::Escape => "escape",
        }
    }
}
#[derive(Serialize)]
struct LogCharacter {
    eno: i16,
    name: String,
    acronym: char,
    color: String,
    hp: i16,
    mp: i16,
    atk: i16,
    tec: i16,
}
impl LogCharacter {
    fn new(ch: &Character) -> LogCharacter {
        LogCharacter {
            eno: ch.eno,
            name: ch.name.clone(),
            acronym: ch.acronym,
            color: format!("{:02x}{:02x}{:02x}", ch.color[0], ch.color[1], ch.color[2]),
            hp: ch.status.hp,
            mp: ch.status.mp,
            atk: ch.status.atk,
            tec: ch.status.tec,
        }
    }
}
#[derive(Serialize)]
struct LogTurn {
    owner: String,
    content: Option<String>,
    skill: Option<(String, Option<String>)>,
    action: Option<String>,
}
impl LogTurn {
    fn make_string(owner: &str, content: &Option<String>, skill: &Option<String>, skill_default: Option<&String>, action: Option<&String>) -> LogTurn {
        let s0 = skill.to_owned().or(skill_default.cloned());
        let s1 = if skill != &None {
            skill_default
        } else { None };
        LogTurn {
            owner: owner.to_string(),
            content: content.to_owned(),
            skill: s0.map(|x| (x, s1.map(|x| x.to_owned()))),
            action: action.cloned(),
        }
    }
    fn make(owner: &str, content: Option<&str>, skill: Option<&str>, skill_default: Option<&str>, action: Option<&str>) -> LogTurn {
        let s0 = skill.or(skill_default);
        let s1 = if skill != None {
            skill_default
        } else { None };
        LogTurn {
            owner: owner.to_string(),
            content: content.map(|x| x.to_string()),
            skill: s0.map(|x| (x.to_string(), s1.map(|x| x.to_string()))),
            action: action.map(|x| x.to_string()),
        }
    }
}
#[derive(Serialize)]
struct Log {
    rule: String,
    version: f32,
    left: LogCharacter,
    right: LogCharacter,
    range: i16,
    escape_range: i16,
    result: String,
    turn: Vec<LogTurn>,
}
struct Battle {
    character: [Character; 2],
    range: i16,
    escape_range: i16,
    result: Option<BattleResult>,
    log: Log,
}
impl Battle {
    fn new(ch0: &Character, ch1: &Character, range: i16, escape_range: i16) -> Battle {
        Battle {
            character: [ch0.to_owned(), ch1.to_owned()],
            range,
            escape_range,
            result: None,
            log: Log { rule: SYSTEM.to_string(), version: 0.2, left: LogCharacter::new(ch0), right: LogCharacter::new(ch1), range, escape_range, result: String::new(), turn: vec![] },
        }
    }
    fn load(eno: [i16; 2]) -> Result<Battle, rusqlite::Error> {
        let conn = Connection::open(common::DATABASE)?;
        // 基幹データ取得
        let (range, escape_range): (i16, i16) = conn.query_row("SELECT start_range,start_escape_range FROM gamerule", [], |row|Ok((row.get(0)?, row.get(1)?)))?;
        let mut character = Vec::new();
        for eno in eno {
            let ch = if eno > 0 {
                // プレイヤー
                let mut character: Character = conn.query_row("SELECT name,acronym,color,word FROM character WHERE eno=?1", params![eno], |row| {
                    Ok(Character {
                        eno,
                        name: row.get(0)?,
                        acronym: row.get::<_, String>(1)?.chars().next().ok_or(rusqlite::Error::InvalidColumnType(1, "character.acronym".to_string(), Type::Text))?,
                        color: row.get(2)?,
                        word: serde_json::from_str(&row.get::<_, String>(3)?).map_err(|_| rusqlite::Error::InvalidColumnType(3, "character.word".to_string(), Type::Text))?,
                        status: Status::new(),
                        skill: Vec::new(),
                    })
                })?;
                let mut stmt = conn.prepare("WITH f AS (SELECT slot,status,skill,skillname,skillword FROM fragment WHERE eno=?1 AND slot<=10) SELECT f.status,s.name,f.skillname,f.skillword,s.type,s.effect FROM f LEFT OUTER JOIN skill s ON f.skill=s.id ORDER BY f.slot ASC")?;
                let result = stmt.query_map(params![eno], |row| {
                    let skill = if let Some(name) = row.get(1)? {
                        Some(Skill::new(
                            name,
                            row.get(2)?,
                            row.get(3)?,
                            Timing::from(row.get::<_, Option<i8>>(4)?.ok_or(rusqlite::Error::InvalidColumnType(4, "skill.type".to_string(), rusqlite::types::Type::Text))?),
                            Command::convert(
                                row.get::<_, Option<_>>(5)?
                                    .ok_or(rusqlite::Error::InvalidColumnType(5, "skill.type".to_string(), rusqlite::types::Type::Text))?
                                ).map_err(|_| rusqlite::Error::InvalidColumnType(5, "skill.effect".to_string(), rusqlite::types::Type::Text)
                            )?,
                        ))
                    } else {
                        None
                    };
                    Ok((
                        Status::from(row.get::<_, [u8; 8]>(0)?),
                        skill,
                    ))
                })?.collect::<Result<Vec<_>, _>>()?;
                for i in result {
                    character.status += i.0;
                    if let Some(skill) = i.1 {
                        character.skill.push((skill, false));
                    }
                }
                character
            } else {
                // NPC
                let mut character = conn.query_row("SELECT name,acronym,color,word,status FROM npc WHERE id=?1", params![-eno], |row| {
                    Ok(Character {
                        eno,
                        name: row.get(0)?,
                        acronym: row.get::<_, String>(1)?.chars().next().ok_or(rusqlite::Error::InvalidColumnType(1, "character.acronym".to_string(), Type::Text))?,
                        color: row.get(2)?,
                        word: serde_json::from_str(&row.get::<_, String>(3)?).map_err(|_| rusqlite::Error::InvalidColumnType(3, "character.word".to_string(), Type::Text))?,
                        status: Status::from(row.get::<_, [u8; 8]>(4)?),
                        skill: Vec::new(),
                    })
                })?;
                // なんでこれLEFT OUTER JOINなの？ INNER JOINでよくない？
                // ああ　あっちはフラグメントだったから、スキルが付いてないやつを取得することもあったのか
                // これは純粋にスキルに対する追加情報だからINNER JOINの方が良い
                let mut stmt = conn.prepare("SELECT s.name,n.name,n.word,s.type,s.effect FROM npc_skill n INNER JOIN skill s ON n.id=?1 AND n.skill=s.id ORDER BY n.slot ASC")?;
                character.skill = stmt.query_map(params![-eno], |row| {
                    Ok((
                        Skill::new(
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            Timing::from(row.get::<_, i8>(3)?),
                            Command::convert(row.get(4)?).map_err(|_| rusqlite::Error::InvalidColumnType(4, "skill.effect".to_string(), rusqlite::types::Type::Text))?,
                        ),
                        false,
                    ))
                })?.collect::<Result<Vec<_>, _>>()?;
                character
            };
            character.push(ch);
        }
        Ok(Battle::new(&character[0], &character[1], range, escape_range))
    }
    fn skill_execute(&mut self, user: usize, timing: Timing) -> Result<bool, String> {
        let once_skill = timing == Timing::Win || timing == Timing::Lose || timing == Timing::Escape;
        let mut skill_id = None;
        let mut action = Vec::new();
        let mut is_attacked = false;
        // スキルを先頭から検索
        for i in 0..self.character[user].skill.len() {
            // タイミングが同一かつ、タイミングが一部のものであれば発動済みでないのを確認
            if self.character[user].skill[i].0.timing == timing && !(once_skill && self.character[user].skill[i].1) {
                if || -> Option<bool> {
                    let mut stack = Vec::<i16>::new();
                    let mut is_complete = false; // スキルが全て完了したかのフラグ
                    let mut end = false;
                    let mut u = user;
                    // スキルを実行していく
                    for f in &self.character[user].skill[i].0.formula {
                        is_complete = end;
                        match *f {
                            Command::Value(value) => { stack.push(value); },
                            Command::Uhp =>  { stack.push(self.character[u].status.hp); },
                            Command::Ump =>  { stack.push(self.character[u].status.mp); },
                            Command::Uatk => { stack.push(self.character[u].status.atk); },
                            Command::Utec => { stack.push(self.character[u].status.tec); },
                            Command::Thp =>  { stack.push(self.character[u ^ 1].status.hp); },
                            Command::Tmp =>  { stack.push(self.character[u ^ 1].status.mp); },
                            Command::Tatk => { stack.push(self.character[u ^ 1].status.atk); },
                            Command::Ttec => { stack.push(self.character[u ^ 1].status.tec); },
                            Command::ValueRange => { stack.push(self.range); },
                            Command::ValueEscape => { stack.push(self.escape_range); },
    
                            Command::Plus => (),
                            Command::Minus =>  { let v = -stack.pop()?; stack.push(v); },
                            Command::Add => { let v = stack.pop()? + stack.pop()?; stack.push(v); },
                            Command::Sub => { let v = stack.pop()? - stack.pop()?; stack.push(v); },
                            Command::Mul => { let v = stack.pop()? * stack.pop()?; stack.push(v); },
                            Command::Div => { let v = stack.pop()? / stack.pop()?; stack.push(v); },
                            Command::Mod => { let v = stack.pop()? % stack.pop()?; stack.push(v); },
                            Command::RandomRange => { let min = stack.pop()?; let v = rand::random::<u16>() % (stack.pop()? - min) as u16; stack.push(v as i16 + min); },
    
                            Command::Cost => {
                                let v = stack.pop()?;
                                if self.character[u].status.mp >= v {
                                    self.character[u].status.mp -= v;
                                    action.push(format!("{} {}", String::from(f.to_owned()), v));
                                } else {
                                    break;
                                }
                            },
                            Command::Range => {
                                if stack.pop()? <= self.range && stack.pop()? >= self.range {
                                    action.push(format!("{} {}", String::from(f.to_owned()), self.range));
                                } else {
                                    break;
                                }
                            },
                            Command::ForceCost => {
                                let v = stack.pop()?;
                                self.character[u].status.mp -= v;
                                if self.character[u].status.mp < 0 {
                                    self.character[u].status.hp += self.character[u].status.mp;
                                }
                                action.push(format!("{} {}", String::from(f.to_owned()), v));
                            },
                            Command::Random => {
                                let v = stack.pop()?;
                                if v > (rand::random::<u16>() % 100) as i16 {
                                    action.push(format!("{} {}", String::from(f.to_owned()), v));
                                } else {
                                    break;
                                }
                            }
                            Command::Break => break,
                            
                            Command::Attack => {
                                let v = stack.pop()?;
                                self.character[u ^ 1].status.hp -= v;
                                is_attacked = true;
                                action.push(format!("{} {}", String::from(f.to_owned()), v));
                            },
                            Command::ForceAttack => {
                                let v = stack.pop()?;
                                self.character[u ^ 1].status.hp -= v;
                                action.push(format!("{} {}", String::from(f.to_owned()), v));
                            },
                            Command::MindAttack => {
                                let v = stack.pop()?;
                                self.character[u ^ 1].status.mp -= v;
                                is_attacked = true;
                                action.push(format!("{} {}", String::from(f.to_owned()), v));
                            },
                            Command::Heal => {
                                let v = stack.pop()?;
                                self.character[u].status.hp += v;
                                action.push(format!("{} {}", String::from(f.to_owned()), v));
                            },
                            Command::SelfDamage => {
                                let v = stack.pop()?;
                                self.character[u].status.hp -= v;
                                action.push(format!("{} {}", String::from(f.to_owned()), v));
                            },
                            Command::Concentrate => {
                                let v = stack.pop()?;
                                self.character[u].status.mp += v;
                                action.push(format!("{} {}", String::from(f.to_owned()), v));
                            },
                            Command::BuffAtk => {
                                let v = stack.pop()?;
                                self.character[u].status.atk += v;
                                action.push(format!("{} {}", String::from(f.to_owned()), v));
                            },
                            Command::BuffTec => {
                                let v = stack.pop()?;
                                self.character[u].status.tec += v;
                                action.push(format!("{} {}", String::from(f.to_owned()), v));
                            },
                            Command::Move => {
                                let v = stack.pop()?;
                                self.range = 0.max(self.range + v);
                                action.push(format!("{} {}", String::from(f.to_owned()), v));
                            },
                            Command::ChangeRange => {
                                let v = stack.pop()?;
                                self.range = v;
                                action.push(format!("{} {}", String::from(f.to_owned()), v));
                            },
                            Command::ChangeEscapeRange => {
                                let v = stack.pop()?;
                                self.escape_range = v;
                                action.push(format!("{} {}", String::from(f.to_owned()), v));
                            },
                            Command::ChangeUser => {
                                u = u ^ 1;
                                action.push(String::from(f.to_owned()));
                            },
                            Command::BreakToEnd => {
                                end = true;
                            },
                        };
                        is_complete = true;
                    }
                    Some(is_complete)
                }().ok_or("式がおかしいよ")? {
                    // 発動済みフラグを立てる
                    self.character[user].skill[i].1 = true;
                    // 発動したスキルのidを保存
                    skill_id = Some(i);
                    // 終了
                    break;
                }
            }
        }
        // ログを生成
        if skill_id != None || !action.is_empty() {
            let defaut_name = String::new();
            let (content, skill, default_name) = if let Some(id) = skill_id {
                (
                    &self.character[user].skill[id].0.word,
                    &self.character[user].skill[id].0.name,
                    &self.character[user].skill[id].0.default_name,
                )
            } else { (&None, &None, &defaut_name) };
            let action = action.join(",");
            self.log.turn.push(LogTurn::make_string(
                get_side(&user),
                content,
                skill,
                Some(default_name),
                if action.is_empty() { None } else { Some(&action) },
            ))
        }
        // 発動したスキルに攻撃が含まれていたら
        if is_attacked {
            // 相手の反応スキルを発動
            self.skill_execute(user ^ 1, Timing::Reactive)
        } else {
            Ok(skill_id.is_some())
        }
    }
    fn check_battle_result(&mut self) -> Result<Option<BattleResult>, String> {
        static CHECK: &str = "<hr>";
        // 戦闘終了判定
        let death_left = self.character[0].status.hp <= 0;
        let death_right = self.character[1].status.hp <= 0;
        if death_left && death_right {
            // どちらもHP0以下なら引き分け
            self.log.turn.push(LogTurn::make(SYSTEM, Some(CHECK), None, None, None));
            Ok(Some(BattleResult::Draw))
        } else if death_left || death_right {
            // どちらかがHP0以下なら勝利判定
            self.log.turn.push(LogTurn::make(SYSTEM, Some(CHECK), None, None, None));
            // スキル発動フラグ
            let mut is_action = false;
            // 勝者側のインデックスを作成
            let winer = death_left as usize;
            // 敗北側から先にスキル発動
            is_action |= self.skill_execute(winer ^ 1, Timing::Lose)?;
            is_action |= self.skill_execute(winer, Timing::Win)?;
            if is_action {
                // どちらかが行動していれば判定をやり直す
                self.check_battle_result()
            } else {
                // どちらとも行動していなければ終了
                Ok(Some(BattleResult::Win(winer)))
            }
        } else if self.range >= self.escape_range {
            // 間合が規定を超えていれば逃走
            self.log.turn.push(LogTurn::make(SYSTEM, Some(CHECK), None, None, None));
            Ok(Some(BattleResult::Escape))
        } else {
            Ok(None)
        }
    }
    fn talk(&mut self, i: usize, key: &str) {
        if let Some(word) = self.character[i].word.get(key) {
            if word != "" {
                self.log.turn.push(LogTurn::make(get_side(&i), Some(word), None, None, None))
            }
        }
    }
    fn reward(&mut self) -> Result<(), String> {
        match self.result {
            Some(BattleResult::Win(i)) => {
                let conn = Connection::open(common::DATABASE).map_err(|err| err.to_string())?;
                // フラグメント移動
                if let Some(fragment) = take_fragment(&conn, self.character[i].eno, self.character[i ^ 1].eno).map_err(|err| err.to_string())? {
                    self.log.turn.push(LogTurn::make_string(SYSTEM, &Some(format!("フラグメント『{}』が奪われました", fragment)), &None, None, None));
                } else {
                    self.log.turn.push(LogTurn::make_string(SYSTEM, &Some(format!("{}の所持数限界のため報酬を省略", self.character[i].name)), &None, None, None));
                }
                Ok(())
            }
            Some(_) => {
                Ok(())
            }
            None => {
                Err("結果が出ていない状態で報酬処理を行おうとしました".to_string())
            }
        }
    }
    fn save_log(&self) -> Result<i64, String> {
        if let Some(result) = &self.result {
            let conn = Connection::open(common::DATABASE).map_err(|err| err.to_string())?;
            let result = result.as_str();
            let log = serde_json::to_string(&self.log).map_err(|err| err.to_string())?;
            // データベースに保存
            conn.execute(
                "INSERT INTO battle(left_eno,right_eno,result,log) VALUES(?1,?2,?3,?4)",
                params![
                    self.character[0].eno,
                    self.character[1].eno,
                    result,
                    log,
                ]
            ).map_err(|err| err.to_string())?;
            Ok(conn.last_insert_rowid())
        } else {
            Err("結果が出ていない状態でログを保存しようとしました".to_string())
        }
    }
}

pub fn take_fragment(conn: &Connection, win: i16, lose: i16) -> Result<Option<String>, rusqlite::Error> {
    // 勝利者がプレイヤー
    if win > 0 {
        // 勝利者のスロットに空きがある場合
        if let Some(slot) = common::get_empty_slot(&conn, win)? {
            if lose > 0 {
                // 敗北者がプレイヤー
                let mut stmt = conn.prepare("SELECT slot,category,name FROM fragment WHERE eno=?1 AND slot<=20")?;
                // 候補の取得
                let result: Vec<(i8, String, String)> = stmt.query_map(params![lose], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                    ))
                })?.collect::<Result<_, _>>()?;
                // 移動対象を決定
                let buf = &result[rand::random::<usize>() % result.len()];
                let t = {
                    if buf.1 == "名前" {
                        if let Some(doll) = result.iter().find(|&x| x.1 == "身代わり") {
                            doll
                        } else {
                            buf
                        }
                    } else { buf }
                };
                // フラグメントの移動
                conn.execute("UPDATE fragment SET eno=?1,slot=?2 WHERE eno=?3 AND slot=?4", params![win, slot, lose, t.0])?;
                Ok(Some(t.2.to_owned()))
            } else {
                // 敗北者がNPC
                let mut stmt = conn.prepare("SELECT weight,category,name,lore,status,skill FROM reward WHERE npc=?1")?;
                // 候補の取得
                let result: Vec<(i32, String, String, String, [u8; 8], Option<i32>)> = stmt.query_map(params![lose * -1], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                })?.collect::<Result<_, _>>()?;
                if !result.is_empty() {
                    let weight = WeightedIndex::new(result.iter().map(|x| x.0).collect::<Vec<_>>()).unwrap();
                    // 移動対象を決定
                    let t = &result[weight.sample(&mut rand::thread_rng())];
                    // 獲得
                    conn.execute("INSERT INTO fragment(eno,slot,category,name,lore,status,skill) VALUES(?1,?2,?3,?4,?5,?6,?7)", params![
                        win, slot, t.1, t.2, t.3, t.4, t.5,
                    ])?;
                    Ok(Some(t.2.to_owned()))
                } else {
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    } else {
        // 勝利者がNPC
        let mut stmt = conn.prepare("SELECT slot,category,name FROM fragment WHERE eno=?1 AND slot<=20")?;
        // 候補の取得
        let result: Vec<(i8, String, String)> = stmt.query_map(params![lose], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
            ))
        })?.collect::<Result<_, _>>()?;
        // 移動対象を決定
        let buf = &result[rand::random::<usize>() % result.len()];
        let t = {
            if buf.1 == "名前" {
                if let Some(doll) = result.iter().find(|&x| x.1 == "身代わり") {
                    doll
                } else {
                    buf
                }
            } else { buf }
        };
        // フラグメントの削除
        conn.execute("DELETE FROM fragment WHERE eno=?1 AND slot=?2", params![lose, t.0])?;
        Ok(Some(t.2.to_owned()))
    }
}
fn get_side(i: &usize) -> &str {
    match i {
        0 => "left",
        1 => "right",
        _ => "",
    }
}

pub fn battle(eno: [i16; 2]) -> Result<(BattleResult, String), String> {
    // 読み込み・初期化
    let mut battle = Battle::load(eno).map_err(|err|err.to_string())?;
    // 処理開始
    // 開始前スキル
    for i in 0..battle.character.len() {
        battle.skill_execute(i, Timing::Start)?;
        // 戦闘終了判定
        battle.result = battle.check_battle_result()?;
    }
    // 戦闘開始時台詞
    for i in 0..battle.character.len() {
        battle.talk(i, "start");
    }
    // 戦闘開始ログ
    battle.log.turn.push(LogTurn::make(SYSTEM, Some("<hr>戦闘開始"), None, None, None));
    // ターン処理
    // もしこの時点で戦闘が終了していればスキップ
    let mut turn = 0;
    while battle.result == None && turn <= 30 {
        turn += 1;
        battle.log.turn.push(LogTurn::make(SYSTEM, Some(&format!("<hr>ターン {}", turn)), None, None, None));
        for i in 0..battle.character.len() {
            battle.skill_execute(i, Timing::Active)?;
            // 戦闘終了判定
            battle.result = battle.check_battle_result()?;
            if battle.result != None {
                break;
            }
        }
    }
    // 戦闘終了時台詞
    battle.log.turn.push(LogTurn::make(SYSTEM, Some("戦闘終了"), None, None, None));
    match battle.result {
        Some(BattleResult::Win(winer)) => {
            // 勝った方の台詞を先に
            battle.talk(winer, "win");
            battle.talk(winer ^ 1, "lose");
            battle.log.result = get_side(&winer).to_string();
            battle.log.turn.push(LogTurn::make((battle.log.result.to_owned() + "-").as_str(), Some(&format!("{} の勝利", battle.character[winer].name)), None, None, None));
        },
        Some(BattleResult::Draw) => {
            for i in 0..battle.character.len() {
                battle.talk(i, "draw");
            }
            battle.log.result = "draw".to_string();
            battle.log.turn.push(LogTurn::make(SYSTEM, Some("引き分け"), None, None, None));
        },
        Some(BattleResult::Escape) => {
            for i in 0..battle.character.len() {
                battle.talk(i, "escape");
            }
            battle.log.result = "escape".to_string();
            battle.log.turn.push(LogTurn::make(SYSTEM, Some("逃走"), None, None, None));
        },
        None => {
            battle.result = Some(BattleResult::Draw);
            for i in 0..battle.character.len() {
                battle.talk(i, "timeover");
            }
            battle.log.result = "draw".to_string();
            battle.log.turn.push(LogTurn::make(SYSTEM, Some("引き分け……時間切れ"), None, None, None));
        },
    }
    // 戦利品処理
    battle.reward()?;
    // ログ保存
    battle.save_log()?;
    // 処理終了
    Ok((battle.result.unwrap(), serde_json::to_string(&battle.log).map_err(|err| err.to_string())?))
}
