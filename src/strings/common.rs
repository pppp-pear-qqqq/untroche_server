use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use awc::Client;
use fancy_regex::Regex;
use rusqlite::{params, Connection};

pub(super) struct Fragment {
    category: String,
    pub name: String,
    lore: String,
    status: [u8; 8],
    skill: Option<i32>,
}
impl Fragment {
    pub fn new(category: String, name: String, lore: String, status: [u8; 8], skill: Option<i32>) -> Fragment {
        Fragment { category, name, lore, status, skill }
    }
}

// ================================
// アプリケーション共通処理
// ================================

pub static DATABASE: &str = "db/strings.db";

pub fn open_database() -> Result<Connection, actix_web::Error> {
    Connection::open(DATABASE).map_err(|err| actix_web::error::ErrorInternalServerError(err))
}

// ログインセッションを確認し、enoを返却する
pub fn session_to_eno(conn: &Connection, session: &str) -> Result<i16, actix_web::Error> {
    let session = session.parse::<i128>().map_err(|err| ErrorBadRequest(err))?;
    match conn.prepare("SELECT eno FROM login_session WHERE id=?1") {
        Ok(mut stmt) => match stmt.query_row(&[&session], |row| Ok(row.get(0)?)) {
            Ok(eno) => Ok(eno),
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(ErrorBadRequest("ログインセッションが無効です")),
            Err(err) => Err(ErrorInternalServerError(err))
        },
        Err(err) => Err(ErrorInternalServerError(err)),
    }
}

// Discordウェブフックに送信
pub async fn send_webhook(eno: i16, content: String) -> Result<(), String> {
    let url: Option<String> = open_database()
        .map_err(|err| err.to_string())?
        .query_row("SELECT webhook FROM user WHERE eno=?1", &[&eno], |row| row.get(0)).map_err(|err| err.to_string())?;
    if let Some(url) = url {
        let client = Client::default();
        client.post(url)
            .insert_header(("Content-Type", "application/json"))
            .send_body(format!(r#"{{"username":"\"Strings\"","avatar_url":"https://game.428.st/uploads/8d6abc63-6420-4580-8bd6-38b982a01632.png","content":"{}"}}"#, content))
            .await
            .map_err(|err| err.to_string())?;
    }
    // ウェブフックが登録されてなくて送信できなかったパターンは成功として扱う
    Ok(())
}

// キャラクターの空きスロットを確認
pub fn get_empty_slot(conn: &Connection, eno: i16) -> Result<Option<i8>, rusqlite::Error> {
    let mut stmt = conn
        .prepare("SELECT slot FROM fragment WHERE eno=?1 ORDER BY slot ASC")?;
    let result = stmt
        .query_map(params![eno], |row| Ok(row.get(0)?))?
        .collect::<Result<Vec<i8>, rusqlite::Error>>()?;
    if result.is_empty() {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    let mut i = 0;
    for slot in result {
        i += 1;
        if i != slot {
            break;
        }
    }
    if i <= 30 {
        Ok(Some(i))
    } else {
        Ok(None)
    }
}

// 空きスロットを検索してフラグメントを追加
pub fn add_fragment(conn: &Connection, eno: i16, fragment: &Fragment) -> Result<bool, rusqlite::Error> {
    Ok(if let Some(slot) = get_empty_slot(&conn, eno)? {
        // 追加
        conn.execute(
            "INSERT INTO fragment(eno,slot,category,name,lore,status,skill) VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![
                eno,
                slot,
                fragment.category,
                fragment.name,
                fragment.lore,
                fragment.status,
                fragment.skill
            ]
        )?;
        true
    } else {
        // 所持数超過により破棄
        let value = calc_fragment_kins(fragment.status);
        conn.execute("UPDATE character SET kins=kins+?1 WHERE eno=?2", params![value, eno])?;
        false
    })
}

pub fn html_special_chars(text: String) -> String {
    text
    .replace('&',"&amp;")
    .replace('"',"&quot;")
    .replace('\'',"&#039;")
    .replace('<',"&lt;")
    .replace('>',"&gt;")
}

pub fn replace_tag(mut text: String, _eno: i16, replace_link: bool) -> Result<String, fancy_regex::Error> {
    // タグ置換
    let re = Regex::new(r"\[(.+)\|(.*)\|\1\]")?;
    while || -> Result<bool, fancy_regex::Error> {
        match re.captures(&text) {
            Ok(Some(caps)) => {
                match &caps[1] {
                    "b" | "bold" => text = re.replace(&text, "<span class='bold'>$2</span>").to_string(),
                    "i" | "italic" => text = re.replace(&text, "<span class='italic'>$2</span>").to_string(),
                    "u" | "underline" => text = re.replace(&text, "<span class='underline'>$2</span>").to_string(),
                    "s" | "linethrough" => text = re.replace(&text, "<span class='linethrough'>$2</span>").to_string(),
                    "large" => text = re.replace(&text, "<span class='large'>$2</span>").to_string(),
                    "small" => text = re.replace(&text, "<span class='small'>$2</span>").to_string(),
                    "rainbow" => text = re.replace(&text, "<span class='rainbow'>$2</span>").to_string(),
                    "image" => {
                        // 世界観スキルを持ち、許可された人だけ可能　そうでないなら未定義と同様の扱い
                        // 一旦誰でも出来るようにしておく
                        text = re.replace(&text, "<img src='$2' style='max-width:30em;max-height:30em'>").to_string();
                    },
                    // 未定義のタグ
                    _ => {
                        // これ二回検索しちゃってる　一旦こうしてはおくけれど
                        // 暇になったらCapturesを使って置換する関数を作ります
                        text = re.replace(&text, "\x11$1|$2|$1\x13").to_string();
                    },
                }
                Ok(true)
            }
            Ok(None) => Ok(false),
            Err(err) => Err(err),
        }
    }()? {}
    // 変換した括弧を元に戻す　元々こんな感じだった場合のことは無視する　制御文字だから多分ないと思う
    text = text.replace('\x11', "[").replace('\x13', "]");
    // 改行をタグに変換する
    text = Regex::new(r"\n|\r\n|\r")?.replace_all(&text, "<br>").to_string();
    // リンク置換モードが有効になっている場合、置換する
    if replace_link {
        text = Regex::new(r"@([\w_]+)@([\w_]+\.[\w_]+)")?.replace_all(&text, "<a href='https://$2/\x11$1' target='_blank'>\x11$1\x11$2</a>").to_string();
        text = Regex::new(r"@(([\w_]+\.)?[\w_]+\.[\w_]+)")?.replace_all(&text, "<a href='https://bsky.app/profile/$1' target='_blank'>\x11$1</a>").to_string();
        text = Regex::new(r"@([\w_]+)")?.replace_all(&text, "<a href='https://twitter.com/$1' target='_blank'>\x11$1</a>").to_string();
        // 変換した@を元に戻す　元々こんな感じだった場合のことは無視する
        text = text.replace('\x11', "@");
    }
    Ok(text)
}

pub fn calc_fragment_kins(status: [u8; 8]) -> i32 {
    let mut kins = 0;
    kins += (status[0] as i32) << 8 | status[1] as i32;
    kins += (status[2] as i32) << 8 | status[3] as i32;
    kins += ((status[4] as i32) << 8 | status[5] as i32) * 3;
    kins += ((status[6] as i32) << 8 | status[7] as i32) * 3;
    return kins.max(0);
}
