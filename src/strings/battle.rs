use std::collections::HashMap;

use rusqlite::{Connection, params};
use serde::Serialize;

use super::common;

static SYSTEM: &str = "strings";

#[derive(Clone)]
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
}
impl Command {
    pub fn from(v: i16) -> Command {
        match v {
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

            _ => Self::Value(v as i16)
        }
    }
    pub fn convert(blob: Vec<u8>) -> Result<Vec<Command>, String> {
        || -> Option<_> {
            let mut skill = Vec::<Command>::new();
            let mut i: usize = 0;
            while i < blob.len() {
                skill.push(Command::from(
                    // この辺も怪しい
                    ((*blob.get(i)? as i16) << 8) | *blob.get(i + 1)? as i16
                ));
                i += 2;
            }
            Some(skill)
        }().ok_or("blobのサイズが不正です".to_string())
    }
    pub fn to_i16(&self) -> i16 {
        match self {
            Self::Value(v) => *v,
            Self::Uhp => -0x1,
            Self::Ump => -0x2,
            Self::Uatk => -0x3,
            Self::Utec => -0x4,
            Self::Thp => -0x5,
            Self::Tmp => -0x6,
            Self::Tatk => -0x7,
            Self::Ttec => -0x8,
            Self::ValueRange => -0x9,
            Self::ValueEscape => -0xa,

            Self::Plus => -0x11,
            Self::Minus => -0x12,
            Self::Add => -0x13,
            Self::Sub => -0x14,
            Self::Mul => -0x15,
            Self::Div => -0x16,
            Self::Mod => -0x17,
            Self::RandomRange => -0x18,

            Self::Cost => -0x21,
            Self::ForceCost => -0x22,
            Self::Range => -0x23,
            Self::Random => -0x24,
            Self::Break => -0x25,

            Self::Attack => -0x31,
            Self::ForceAttack => -0x32,
            Self::MindAttack => -0x33,
            Self::Heal => -0x34,
            Self::SelfDamage => -0x35,
            Self::Concentrate => -0x36,
            Self::BuffAtk => -0x37,
            Self::BuffTec => -0x38,
            Self::Move => -0x39,
            Self::ChangeRange => -0x3a,
            Self::ChangeEscapeRange => -0x3b,
            Self::ChangeUser => -0x3c,
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Self::Value(v) => v.to_string(),
            Self::Uhp => "HP".to_string(),
            Self::Ump => "MP".to_string(),
            Self::Uatk => "ATK".to_string(),
            Self::Utec => "TEC".to_string(),
            Self::Thp => "相手HP".to_string(),
            Self::Tmp => "相手MP".to_string(),
            Self::Tatk => "相手ATK".to_string(),
            Self::Ttec => "相手TEC".to_string(),
            Self::ValueRange => "間合値".to_string(),
            Self::ValueEscape => "逃走値".to_string(),
            Self::Plus => "正".to_string(),
            Self::Minus => "負".to_string(),
            Self::Add => "+".to_string(),
            Self::Sub => "-".to_string(),
            Self::Mul => "*".to_string(),
            Self::Div => "/".to_string(),
            Self::Mod => "%".to_string(),
            Self::RandomRange => "~".to_string(),
            Self::Cost => "消耗".to_string(),
            Self::ForceCost => "強命消耗".to_string(),
            Self::Range => "間合".to_string(),
            Self::Random => "確率".to_string(),
            Self::Break => "中断".to_string(),
            Self::Attack => "攻撃".to_string(),
            Self::ForceAttack => "貫通攻撃".to_string(),
            Self::MindAttack => "精神攻撃".to_string(),
            Self::Heal => "回復".to_string(),
            Self::SelfDamage => "自傷".to_string(),
            Self::Concentrate => "集中".to_string(),
            Self::BuffAtk => "ATK変化".to_string(),
            Self::BuffTec => "TEC変化".to_string(),
            Self::Move => "移動".to_string(),
            Self::ChangeRange => "間合変更".to_string(),
            Self::ChangeEscapeRange => "逃走ライン".to_string(),
            Self::ChangeUser => "対象変更".to_string(),
        }
    }
}
impl Serialize for Command {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        if let Self::Value(v) = &self {
            serializer.serialize_i16(*v)
        } else {
            serializer.serialize_str(&self.to_string())
        }
    }
}

enum SkillResult {
    Ok,
    Fail,
}

#[derive(PartialEq,Clone)]
#[allow(dead_code)]
pub enum Timing {
    Active,
    Reactive,
    Start,
    Win,
    Lose,
    None,
}
impl Timing {
    pub fn from(v: i8) -> Timing {
        match v {
            0 => Self::Active,
            1 => Self::Reactive,
            2 => Self::Start,
            3 => Self::Win,
            4 => Self::Lose,
            _ => Self::None,
        }
    }
    // pub fn to_i8(&self) -> i8 {
    //     match self {
    //         Self::Active => 0,
    //         Self::Reactive => 1,
    //         Self::Start => 2,
    //         Self::Win => 3,
    //         Self::Lose => 4,
    //         Self::None => 5,
    //     }
    // }
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
            Self::None => "無感",
        })
    }
}

#[derive(Clone)]
struct Skill {
    name: String,
    word: String,
    timing: Timing,
    formula: Vec<Command>,
}
impl Skill {
    fn new(name: String, word: String, timing: Timing, formula: Vec<Command>) -> Skill {
        Skill { name, word, timing, formula }
    }
}

#[derive(Clone, Copy)]
struct Status {
    hp: i16,
    mp: i16,
    atk: i16,
    tec: i16,
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
enum BattleResult {
    Win(usize),
    Draw,
    Escape,
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
    fn new(ch: Character) -> LogCharacter {
        LogCharacter {
            eno: ch.eno,
            name: ch.name,
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
    skill: Option<String>,
    action: Option<String>,
}
impl LogTurn {
    fn make(owner: &str, content: Option<&str>, skill: Option<&str>, action: Option<&str>) -> LogTurn {
        LogTurn {
            owner: owner.to_string(),
            content: content.map(|x| x.to_string()),
            skill: skill.map(|x| x.to_string()),
            action: action.map(|x| x.to_string()),
        }
    }
}
#[derive(Serialize)]
pub struct Log {
    rule: String,
    version: f32,
    left: LogCharacter,
    right: LogCharacter,
    range: i16,
    escape_range: i16,
    pub result: String,
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
    fn new(ch0: Character, ch1: Character, range: i16, escape_range: i16) -> Battle {
        Battle {
            character: [ch0.clone(), ch1.clone()],
            range,
            escape_range,
            result: None,
            log: Log { rule: SYSTEM.to_string(), version: 0.1, left: LogCharacter::new(ch0), right: LogCharacter::new(ch1), range, escape_range, result: String::new(), turn: vec![] },
        }
    }
    fn load(ch0: i16, ch1: i16) -> Result<Battle, rusqlite::Error> {
        let conn = Connection::open(common::DATABASE)?;
        // かならず列が2であることから、まとめて処理するように
        let mut stmt = conn.prepare("SELECT eno,name,acronym,color,word FROM character WHERE eno IN(?1,?2)")?;
        let result: &mut Vec<Character> = &mut stmt.query_map(
            params![ch0, ch1],
            |row| {
                Ok(Character {
                    eno: row.get(0)?,
                    name: row.get(1)?,
                    acronym: row.get::<_, String>(2)?.chars().nth(0).ok_or(rusqlite::Error::InvalidColumnType(0, "character.acronym".to_string(), rusqlite::types::Type::Text))?,
                    color: row.get(3)?,
                    word: serde_json::from_str(&row.get::<_, String>(4)?).map_err(|_|rusqlite::Error::InvalidColumnType(0, "character.word".to_string(), rusqlite::types::Type::Text))?,
                    status: Status { hp: 0, mp: 0, atk: 0, tec: 0 },
                    skill: Vec::new(),
                })
            }
        )?.collect::<Result<Vec<_>, _>>()?;
        // スキル・ステータス登録処理
        for ch in &mut *result {
            let mut stmt = conn.prepare("WITH f AS (SELECT slot,status,skill,skillname,skillword FROM fragment WHERE eno=?1 AND slot<=10) SELECT f.status,f.skillname,f.skillword,s.name,s.type,s.effect FROM f LEFT OUTER JOIN skill s ON f.skill=s.id ORDER BY f.slot ASC")?;
            let result = stmt.query_map(params![ch.eno], |row| {
                // スキル名を獲得、NOT NULLなのでこれがNoneの時はスキルなし
                let skill : Option<String> = row.get(3)?;
                Ok((
                    row.get::<_, [u8; 8]>(0)?,  // ステータス
                    match skill {   // スキル
                        Some(s) => Some((
                            match row.get::<_, Option<String>>(1)? {    // スキル名
                                Some(v) => if v == "" { s } else { v },
                                None => s,
                            },
                            row.get::<_, Option<String>>(2)?.unwrap_or(String::new()),  // スキル発動台詞
                            Timing::from(row.get::<_, Option<i8>>(4)?.ok_or(rusqlite::Error::InvalidColumnType(0, "skill.type".to_string(), rusqlite::types::Type::Text))?),    // 発動タイミング
                            Command::convert(row.get::<_, Option<Vec<u8>>>(5)?.ok_or(rusqlite::Error::InvalidColumnType(0, "skill.type".to_string(), rusqlite::types::Type::Text))?).map_err(|_| rusqlite::Error::InvalidColumnType(0, "skill.effect".to_string(), rusqlite::types::Type::Text))?,  // スキル式
                        )),
                        None => None,
                    }
                ))
            })?.collect::<Result<Vec<_>, _>>()?;
            // ステータス反映
            for r in result {
                ch.status.hp += (r.0[0] as i16) << 8 | r.0[1] as i16;
                ch.status.mp += (r.0[2] as i16) << 8 | r.0[3] as i16;
                ch.status.atk += (r.0[4] as i16) << 8 | r.0[5] as i16;
                ch.status.tec += (r.0[6] as i16) << 8 | r.0[7] as i16;
                if let Some(s) = r.1 {
                    ch.skill.push((Skill::new(s.0, s.1, s.2, s.3), false));
                }
            }
        }
        // デバッグ用出力
        // println!("{}\nHP:{}, MP:{}, ATK:{}, TEC{}", result[0].name, result[0].status.hp, result[0].status.mp, result[0].status.atk, result[0].status.tec);
        // for s in &result[0].skill { println!("{}", s.0.name); }
        // println!("{}\nHP:{}, MP:{}, ATK:{}, TEC{}", result[1].name, result[1].status.hp, result[1].status.mp, result[1].status.atk, result[1].status.tec);
        // for s in &result[1].skill { println!("{}", s.0.name); }
        // 基幹データ取得
        let (range, escape_range): (i16, i16) = conn.query_row("SELECT start_range,start_escape_range FROM gamerule", [], |row|Ok((row.get(0)?, row.get(1)?)))?;
        // 返却
        if ch0 == result[0].eno {
            Ok(Battle::new(result[0].clone(), result[1].clone(), range, escape_range))
        } else {
            Ok(Battle::new(result[1].clone(), result[0].clone(), range, escape_range))
        }
    }

    fn skill_execute(&mut self, user: usize, timing: Timing) -> Result<bool, String> {
        static ERR: &str = "式がおかしいよ";
        let mut is_complete = false; // スキルが全て完了したかのフラグ
        let mut is_attacked = false;
        let once_skill = timing == Timing::Win || timing == Timing::Lose;
        let mut action = Vec::new();
        let mut skill_id: Option<usize> = None;
        // スキルを先頭から検索
        for i in 0..self.character[user].skill.len() {
            // タイミングが同一
            if self.character[user].skill[i].0.timing == timing {
                // 勝利・敗北時にはそのスキルが一度も発動していないのを確認
                if once_skill && self.character[user].skill[i].1 {
                    continue;
                }
                // 完了フラグを初期化
                is_complete = true;
                let mut stack = Vec::<i16>::new();
                let mut u = user;
                // スキルを実行していく
                for f in &self.character[user].skill[i].0.formula {
                    if let SkillResult::Fail = match *f {
                        Command::Value(value) => { stack.push(value); SkillResult::Ok },
                        Command::Uhp =>  { stack.push(self.character[u].status.hp); SkillResult::Ok },
                        Command::Ump =>  { stack.push(self.character[u].status.mp); SkillResult::Ok },
                        Command::Uatk => { stack.push(self.character[u].status.atk); SkillResult::Ok },
                        Command::Utec => { stack.push(self.character[u].status.tec); SkillResult::Ok },
                        Command::Thp =>  { stack.push(self.character[u ^ 1].status.hp); SkillResult::Ok },
                        Command::Tmp =>  { stack.push(self.character[u ^ 1].status.mp); SkillResult::Ok },
                        Command::Tatk => { stack.push(self.character[u ^ 1].status.atk); SkillResult::Ok },
                        Command::Ttec => { stack.push(self.character[u ^ 1].status.tec); SkillResult::Ok },
                        Command::ValueRange => { stack.push(self.range); SkillResult::Ok },
                        Command::ValueEscape => { stack.push(self.escape_range); SkillResult::Ok },

                        Command::Plus => SkillResult::Ok,
                        Command::Minus =>  { let v = -stack.pop().ok_or(ERR)?; stack.push(v); SkillResult::Ok },
                        Command::Add => { let v = stack.pop().ok_or(ERR)? + stack.pop().ok_or(ERR)?; stack.push(v); SkillResult::Ok },
                        Command::Sub => { let v = stack.pop().ok_or(ERR)? - stack.pop().ok_or(ERR)?; stack.push(v); SkillResult::Ok },
                        Command::Mul => { let v = stack.pop().ok_or(ERR)? * stack.pop().ok_or(ERR)?; stack.push(v); SkillResult::Ok },
                        Command::Div => { let v = stack.pop().ok_or(ERR)? / stack.pop().ok_or(ERR)?; stack.push(v); SkillResult::Ok },
                        Command::Mod => { let v = stack.pop().ok_or(ERR)? % stack.pop().ok_or(ERR)?; stack.push(v); SkillResult::Ok },
                        Command::RandomRange => { let min = stack.pop().ok_or(ERR)?; let v = rand::random::<u16>() % (stack.pop().ok_or(ERR)? - min) as u16; stack.push(v as i16 + min); SkillResult::Ok },

                        Command::Cost => {
                            let v = stack.pop().ok_or(ERR)?;
                            if self.character[u].status.mp >= v {
                                self.character[u].status.mp -= v;
                                action.push((Command::Cost, v));
                                SkillResult::Ok
                            } else {
                                SkillResult::Fail
                            }
                        },
                        Command::Range => {
                            if stack.pop().ok_or(ERR)? <= self.range && stack.pop().ok_or(ERR)? >= self.range {
                                action.push((Command::Range, 0));
                                SkillResult::Ok
                            } else {
                                SkillResult::Fail
                            }
                        },
                        Command::ForceCost => {
                            let v = stack.pop().ok_or(ERR)?;
                            self.character[u].status.mp -= v;
                            if self.character[u].status.mp < 0 {
                                self.character[u].status.hp += self.character[u].status.mp;
                            }
                            action.push((Command::ForceCost, v));
                            SkillResult::Ok
                        },
                        Command::Random => {
                            let v = stack.pop().ok_or(ERR)?;
                            if v > (rand::random::<u16>() % 100) as i16 {
                                action.push((Command::Random, v));
                                SkillResult::Ok
                            } else {
                                SkillResult::Fail
                            }
                        }
                        Command::Break => SkillResult::Fail,
                        
                        Command::Attack => {
                            let v = stack.pop().ok_or(ERR)?;
                            self.character[u ^ 1].status.hp -= v;
                            action.push((Command::Attack, v));
                            SkillResult::Ok
                        },
                        Command::ForceAttack => {
                            let v = stack.pop().ok_or(ERR)?;
                            self.character[u ^ 1].status.hp -= v;
                            action.push((Command::ForceAttack, v));
                            SkillResult::Ok
                        },
                        Command::MindAttack => {
                            let v = stack.pop().ok_or(ERR)?;
                            self.character[u ^ 1].status.mp -= v;
                            action.push((Command::MindAttack, v));
                            SkillResult::Ok
                        },
                        Command::Heal => {
                            let v = stack.pop().ok_or(ERR)?;
                            self.character[u].status.hp += v;
                            action.push((Command::Heal, v));
                            SkillResult::Ok
                        },
                        Command::SelfDamage => {
                            let v = stack.pop().ok_or(ERR)?;
                            self.character[u].status.hp -= v;
                            action.push((Command::SelfDamage, v));
                            SkillResult::Ok
                        },
                        Command::Concentrate => {
                            let v = stack.pop().ok_or(ERR)?;
                            self.character[u].status.mp += v;
                            action.push((Command::Concentrate, v));
                            SkillResult::Ok
                        },
                        Command::BuffAtk => {
                            let v = stack.pop().ok_or(ERR)?;
                            self.character[u].status.atk += v;
                            action.push((Command::BuffAtk, v));
                            SkillResult::Ok
                        },
                        Command::BuffTec => {
                            let v = stack.pop().ok_or(ERR)?;
                            self.character[u].status.tec += v;
                            action.push((Command::BuffTec, v));
                            SkillResult::Ok
                        },
                        Command::Move => {
                            let v = stack.pop().ok_or(ERR)?;
                            self.range = 0.max(self.range + v);
                            action.push((Command::Move, v));
                            SkillResult::Ok
                        },
                        Command::ChangeRange => {
                            let v = stack.pop().ok_or(ERR)?;
                            self.range = v;
                            action.push((Command::ChangeRange, v));
                            SkillResult::Ok
                        },
                        Command::ChangeEscapeRange => {
                            let v = stack.pop().ok_or(ERR)?;
                            self.escape_range = v;
                            action.push((Command::ChangeEscapeRange, v));
                            SkillResult::Ok
                        },
                        Command::ChangeUser => {
                            u = u ^ 1;
                            action.push((Command::ChangeUser, 0));
                            SkillResult::Ok
                        }
                    } {
                        // もしスキルが失敗したら、完了フラグを折ってスキルを中断する
                        is_complete = false;
                        break;
                    } else if let Command::Attack = f {
                        // 攻撃コマンドだった場合、攻撃済みフラグを立てる
                        // 本当は攻撃のたびに反応してもいいんだけど、台詞の処理が大変になってしまうので後で1回だけ発動するように
                        is_attacked = true;
                    };
                }
                // スキルが完了していたら
                if is_complete {
                    // 発動済みフラグを立てる
                    self.character[user].skill[i].1 = true;
                    skill_id = Some(i);
                    // ここで終了
                    break;
                }
            }
        }
        // 台詞
        let mut action_text = String::new();
        for a in action {
            match a.0 {
                Command::Attack |
                Command::ForceAttack |
                Command::MindAttack |
                Command::Heal |
                Command::SelfDamage |
                Command::Concentrate |
                Command::BuffAtk |
                Command::BuffTec |
                Command::Move |
                Command::ChangeRange |
                Command::ChangeEscapeRange => action_text += &format!(",{} {}", a.0.to_string(), a.1),
                Command::ChangeUser => action_text += &format!(",{}", a.0.to_string()),
                _ => (),
            }
        }
        let log = if let Some(i) = skill_id {
            (
                if &self.character[user].skill[i].0.word != "" {
                    Some(self.character[user].skill[i].0.word.as_str())
                } else { None },
                Some(self.character[user].skill[i].0.name.as_str())
            )
        } else { (None, None) };
        let action_text = if action_text != "" { Some(&action_text[1..]) } else { None };
        // いずれかがNoneでない場合、発言生成
        if action_text != None || log.1 != None || log.0 != None {
            self.log.turn.push(LogTurn::make(
                get_side(&user),
                log.0,
                log.1,
                action_text,
            ));
        }
        // 発動したスキルに攻撃が含まれていたら
        if is_attacked {
            // 相手の反応スキルを発動
            self.skill_execute(user ^ 1, Timing::Reactive)
        } else {
            Ok(is_complete)
        }
    }
}

fn check_battle_result(battle: &mut Battle) -> Result<Option<BattleResult>, String> {
    static CHECK: &str = "<hr>";
    // 戦闘終了判定
    let death_left = battle.character[0].status.hp <= 0;
    let death_right = battle.character[1].status.hp <= 0;
    println!("H {}, M {}, A {}, T {} ... H {}, M {}, A {}, T {}", battle.character[0].status.hp, battle.character[0].status.mp, battle.character[0].status.atk, battle.character[0].status.tec, battle.character[1].status.hp, battle.character[1].status.mp, battle.character[1].status.atk, battle.character[1].status.tec);
    if death_left && death_right {
        // どちらもHP0以下なら引き分け
        battle.log.turn.push(LogTurn::make(SYSTEM, Some(CHECK), None, None));
        Ok(Some(BattleResult::Draw))
    } else if death_left || death_right {
        // どちらかがHP0以下なら勝利判定
        battle.log.turn.push(LogTurn::make(SYSTEM, Some(CHECK), None, None));
        // スキル発動フラグ
        let mut is_action = false;
        // 勝者側のインデックスを作成
        let winer = death_left as usize;
        // 敗北側から先にスキル発動
        is_action |= battle.skill_execute(winer ^ 1, Timing::Lose)?;
        is_action |= battle.skill_execute(winer, Timing::Win)?;
        if is_action {
            // どちらかが行動していれば判定をやり直す
            check_battle_result(battle)
        } else {
            // どちらとも行動していなければ終了
            Ok(Some(BattleResult::Win(winer)))
        }
    } else if battle.range >= battle.escape_range {
        // 間合が規定を超えていれば逃走
        battle.log.turn.push(LogTurn::make(SYSTEM, Some(CHECK), None, None));
        Ok(Some(BattleResult::Escape))
    } else {
        Ok(None)
    }
}

fn get_side(i: &usize) -> &str {
    match i {
        0 => "left",
        1 => "right",
        _ => "",
    }
}

fn talk(battle: &mut Battle, i: usize, key: &str) {
    if let Some(word) = battle.character[i].word.get(key) {
        if word != "" {
            battle.log.turn.push(LogTurn::make(get_side(&i), Some(word), None, None))
        }
    }
}

pub fn battle(ch0_eno: i16, ch1_eno: i16) -> Result<Log, String> {
    // 読み込み・初期化
    let mut battle = Battle::load(ch0_eno, ch1_eno).map_err(|err|err.to_string())?;
    // 処理開始
    // 開始前スキル
    for i in 0..battle.character.len() {
        battle.skill_execute(i, Timing::Start)?;
        // 戦闘終了判定
        battle.result = check_battle_result(&mut battle)?;
    }
    // 戦闘開始時台詞
    for i in 0..battle.character.len() {
        talk(&mut battle, i, "start");
    }
    // 戦闘開始ログ
    battle.log.turn.push(LogTurn::make(SYSTEM, Some("<hr>戦闘開始"), None, None));
    // ターン処理
    // もしこの時点で戦闘が終了していればスキップ
    let mut turn = 1;
    while battle.result == None && turn <= 30 {
        battle.log.turn.push(LogTurn::make(SYSTEM, Some(&format!("<hr>ターン {}", turn)), None, None));
        for i in 0..battle.character.len() {
            battle.skill_execute(i, Timing::Active)?;
            // 戦闘終了判定
            battle.result = check_battle_result(&mut battle)?;
        }
        turn += 1;
    }
    // 戦闘終了時台詞
    battle.log.turn.push(LogTurn::make(SYSTEM, Some("戦闘終了"), None, None));
    match battle.result {
        Some(BattleResult::Win(winer)) => {
            // 勝った方の台詞を先に
            talk(&mut battle, winer, "win");
            talk(&mut battle, winer ^ 1, "lose");
            battle.log.result = get_side(&winer).to_string();
            battle.log.turn.push(LogTurn::make(get_side(&winer), Some(&format!("{} の勝利", battle.character[winer].name)), None, None));
        },
        Some(BattleResult::Draw) => {
            for i in 0..battle.character.len() {
                talk(&mut battle, i, "draw");
            }
            battle.log.result = "draw".to_string();
            battle.log.turn.push(LogTurn::make(SYSTEM, Some("引き分け"), None, None));
        },
        Some(BattleResult::Escape) => {
            for i in 0..battle.character.len() {
                talk(&mut battle, i, "escape");
            }
            battle.log.result = "escape".to_string();
            battle.log.turn.push(LogTurn::make(SYSTEM, Some("逃走"), None, None));
        },
        None => {
            battle.result = Some(BattleResult::Draw);
            for i in 0..battle.character.len() {
                talk(&mut battle, i, "timeover");
            }
            battle.log.result = "draw".to_string();
            battle.log.turn.push(LogTurn::make(SYSTEM, Some("引き分け……時間切れ"), None, None));
        },
    }
    // 処理終了
    Ok(battle.log)
}
