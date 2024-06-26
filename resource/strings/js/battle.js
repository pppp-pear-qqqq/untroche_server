var rule = null;
var version = null;

class Status {
	constructor(raw) {
		this.hp = Number(raw['hp']);
		this.mp = Number(raw['mp']);
		this.atk = Number(raw['atk']);
		this.tec = Number(raw['tec']);
	}
};
class Character {
	constructor(raw, elem) {
		this.eno = raw['eno'];
		this.name = raw['name'];
		this.acronym = raw['acronym'];
		this.color = `#${raw['color']}`;
		if (version < 1){
			this.start_status = new Status(raw);
		} else {
			this.start_status = new Status(raw['status']);
		}
		this.status = structuredClone(this.start_status);
		this.displays = elem.getElementsByTagName('div');
		this.displays[0].firstElementChild.style.backgroundColor = this.color;
		this.displays[1].firstElementChild.style.backgroundColor = this.color;
	}
	update() {
		this.displays[0].dataset.value = this.status.hp;
		this.displays[1].dataset.value = this.status.mp;
		this.displays[2].dataset.value = this.status.atk;
		this.displays[3].dataset.value = this.status.tec;
		if (this.start_status.hp !== 0) this.displays[0].firstElementChild.style.width = `calc(100% * ${this.status.hp} / ${this.start_status.hp})`;
		else this.displays[0].firstElementChild.style.width = 'calc(100%)';
		if (this.start_status.mp !== 0) this.displays[1].firstElementChild.style.width = `calc(100% * ${this.status.mp} / ${this.start_status.mp})`;
		else this.displays[1].firstElementChild.style.width = 'calc(100%)';
	}
};
/**
 * 
 * @param {string} text 
 * @param {number} chance 
 */
function hide_text(text, chance) {
	if (!chance) chance = 0;
	for (let i = 0; i < text.length; ++i) {
		if (Math.random() < chance && /[^a-zA-Z<>"'=/ ─]/.test(text[i])) {
			text = `${text.substring(0, i)}█${text.substring(i + 1)}`;
		}
	}
	return text;
}
class Battle {
	constructor(raw) {
		console.log(raw);
		rule = raw['rule'];
		version = raw['version'];
		this.auto = null;
		if (rule === "strings") {
			if (version < 2) {
				// 要素取得
				this.display = document.getElementById('play_battle');
				this.elem_log = this.display.querySelector('.log');
				const scroll = this.elem_log.parentNode;
				scroll.onclick = () => this.next(this);
				this.elem_range = this.display.querySelector('.range>.range');
				// 初期設定
				if (version < 1) 
					this.character = [new Character(raw['left'], this.display.querySelector('.data.left')), new Character(raw['right'], this.display.querySelector('.data.right'))];
				else if (version < 2)
					this.character = [new Character(raw['character'][0], this.display.querySelector('.data.left')), new Character(raw['character'][1], this.display.querySelector('.data.right'))];
				this.range = Number(raw['range']);
				this.escape_range = Number(raw['escape_range']);
				this.log = raw['turn'];
				this.now = 0;
				// 表示
				this.display.querySelector('.name.left').innerText = this.character[0].name;
				this.display.querySelector('.name.right').innerText = this.character[1].name;
				this.display.querySelector('.range>.acronym.left').innerText = this.character[0].acronym;
				this.display.querySelector('.range>.acronym.right').innerText = this.character[1].acronym;
				this.update();
				this.display.classList.remove('hide');
			} else {
				alertify.error('現在の戦闘再生システムはこのログ形式に対応していません。<br>スーパーリロードしても解決しない場合は運営にご報告ください。')
				delete this;
			}
		}
	}
	update() {
		this.elem_range.style.width = `min(calc(24em * ${this.range} / ${this.escape_range} + 2em), 24em)`
		this.elem_range.innerText = this.range;
		this.character[0].update();
		this.character[1].update();
	}
	/**
	 * 
	 * @param {'left'|'right'} side 
	 */
	background(side) {
		switch (side) {
			case 'left': this.background_left = this.character[0].color; break;
			case 'right': this.background_right = this.character[1].color; break;
		}
		if (this.background_left !== undefined && this.background_right !== undefined)
			this.display.style.background = `linear-gradient(to right, color-mix(in srgb,${this.background_left},#404040 75%) 0% 40%, color-mix(in srgb,${this.background_right},#404040 75%) 60% 100%)`;
		else if (this.background_left !== undefined)
			this.display.style.background = `linear-gradient(to right, color-mix(in srgb,${this.background_left},#404040 75%) 0% 90%, #404040 100%)`;
		else if (this.background_right !== undefined)
			this.display.style.background = `linear-gradient(to left, color-mix(in srgb,${this.background_right},#404040 75%) 0% 90%, #404040 100%)`;
	}
	async close() {
		this.display.classList.add('hide');
		this.display.classList.remove('manaita', 'rainbow');
		this.display.style.background = '#404040';
		this.elem_log.replaceChildren();
		if (this.auto !== null) {
			clearInterval(battle.auto);
			this.auto = null;
		}
	}
	/**
	 * @param {Battle} battle 
	 */
	async next(battle) {
		if (rule === 'strings') {
			if (version < 2) {
				// 現在ターンを取得
				const turn = battle.log[battle.now++];
				// ターンが終了していれば（ログが無ければ）終了
				if (turn === undefined) {
					battle.close();
					return true;
				};
				// ログ作成
				const div = document.createElement('div');
				let actor = null;
				let world = false;
				// 表示位置の決定・行動者の表示
				switch (turn['owner']) {
					case 'strings': {
						div.classList.add('p_center');
					} break;
					case 'left': case 'left-': {
						actor = 0;
						div.classList.add('p_left');
						if ((version < 1 && battle.log[battle.now] !== undefined) || turn['owner'].slice(-1) !== '-') {
							const p = document.createElement('p');
							p.innerText = hide_text(`${battle.character[actor].name}の行動`, this.text_hide);
							div.appendChild(p);
						}
					} break;
					case 'right': case 'right-': {
						actor = 1;
						div.classList.add('p_right');
						if ((version < 1 && battle.log[battle.now] !== undefined) || turn['owner'].slice(-1) !== '-') {
							const p = document.createElement('p');
							p.innerText = hide_text(`${battle.character[actor].name}の行動`, this.text_hide);
							div.appendChild(p);
						}
					} break;
					case 'world-left' :{
						actor = 0;
						div.classList.add('p_left');
						this.background('left');
						if (turn['action'] === '回復(x),集中(x),ATK変化(x),TEC変化(x)のスキルを末尾に追加<br>xの値は「むねがちいさい」フラグメント所持数により決定') {
							this.display.classList.add('manaita');
						} else if (turn['action'] === '████████') {
							this.text_hide = 0;
						} else if (turn['action'] === '間合条件無効化 + 虹霓') {
							this.elem_log.classList.add('rainbow');
						}
						world = true;
					} break;
					case 'world-right' :{
						actor = 1;
						div.classList.add('p_right');
						this.background('right');
						if (turn['action'] === '回復(x),集中(x),ATK変化(x),TEC変化(x)のスキルを末尾に追加<br>xの値は「むねがちいさい」フラグメント所持数により決定') {
							this.display.classList.add('manaita');
						} else if (turn['action'] === '████████') {
							console.log('world activate');
							this.text_hide = 0;
						} else if (turn['action'] === '間合条件無効化 + 虹霓') {
							this.elem_log.classList.add('rainbow');
						}
						world = true;
					} break;
				}
				// 発言内容等
				if (turn['content'] !== null && turn['content'] !== undefined) {
					const p = document.createElement('p');
					p.innerHTML = hide_text(turn['content'], this.text_hide);
					div.appendChild(p);
				}
				// スキル名
				if (turn['skill'] !== null && turn['skill'] !== undefined) {
					if (version < 0.2) {
						const p = document.createElement('p');
						p.className = 'skill';
						p.innerHTML = hide_text(turn['skill'], this.text_hide);
						div.appendChild(p);
					} else {
						const p = document.createElement('p');
						p.className = 'skill';
						p.innerHTML = hide_text(turn['skill'][0], this.text_hide);
						if (turn['skill'][1] !== null)
							p.innerHTML += `<span class="tip">(${hide_text(turn['skill'][1], this.text_hide)})<span>`;
						div.appendChild(p);
					}
				}
				// スキル効果
				if (turn['action'] !== null && turn['action'] !== undefined) {
					if (world) {
						const p = document.createElement('p');
						p.innerHTML = turn['action'];
						div.appendChild(p);
					} else {
						turn['action'].split(',').forEach(action => {
							const a = action.split(' ');
							let pre = null;
							let body = null;
							let value = Number(a[1]);
							switch (a[0]) {
								case '消耗': {
									pre = `${battle.character[actor].name}は構える`
									body = `MPを<span class="special">${value}</span>消費`;
									battle.character[actor].status.mp -= value;
								} break;
								case '間合': {
									pre = '間合判定'
									body = `<span class="special">${value}</span> ── 適正`;
								} break;
								case '強命消耗': {
									pre = `${battle.character[actor].name}は構える`
									body = `MPを<span class="special">${value}</span>消費`;
									battle.character[actor].status.mp -= value;
									const mp = battle.character[actor].status.mp;
									if (mp < 0) {
										battle.character[actor].status.hp += mp;
										body += ` <span class="minus">${-mp}</span>のダメージ`
									}
								} break;
								case '確率': {
									pre = '確率判定'
									body = `<span class="special">${value}</span>% ── 成功`;
								} break;
								case '攻撃': {
									pre = `${battle.character[actor ^ 1].name}への攻撃`
									body = `<span class="minus">${value}</span>のダメージ`;
									battle.character[actor ^ 1].status.hp -= value;
								} break;
								case '貫通攻撃': {
									pre = `${battle.character[actor ^ 1].name}への攻撃`
									body = `<span class="minus">${value}</span>のダメージ`;
									battle.character[actor ^ 1].status.hp -= value;
								} break;
								case '精神攻撃': {
									pre = `${battle.character[actor ^ 1].name}への精神攻撃`
									body = `MPに<span class="special">${value}</span>のダメージ`;
									battle.character[actor ^ 1].status.mp -= value;
								} break;
								case '回復': {
									pre = `${battle.character[actor].name}の傷が癒える`
									body = `<span class="plus">${value}</span>回復した`;
									battle.character[actor].status.hp += value;
								} break;
								case '自傷': {
									pre = `${battle.character[actor].name}に傷が生まれる`
									body = `<span class="minus">${value}</span>のダメージ`;
									battle.character[actor].status.hp -= value;
								} break;
								case '集中': {
									pre = `${battle.character[actor].name}は集中する`
									body = `MPが<span class="plus">${value}</span>増加`;
									battle.character[actor].status.mp += value;
								} break;
								case 'ATK変化': {
									pre = `${battle.character[actor].name}の気迫が揺れる`
									body = `ATKが<span class="special">${value}</span>変化した`;
									battle.character[actor].status.atk += value;
								} break;
								case 'TEC変化': {
									pre = `${battle.character[actor].name}の目つきが変わる`
									body = `TECが<span class="special">${value}</span>変化した`;
									battle.character[actor].status.tec += value;
								} break;
								case '移動': {
									battle.range = Math.max(battle.range + value, 0);
									body = `<span class="special">${value}</span>移動し、間合が<span class="special">${battle.range}</span>になった`;
								} break;
								case '間合変更': {
									battle.range = value;
									body = `構え直し、間合が<span class="special">${battle.range}</span>になった`;
								} break;
								case '逃走ライン': {
									battle.escape_range = value;
									body = `逃走扱いとなる間合が<span class="special">${battle.escape_range}</span>に設定された`;
								} break;
								case '対象変更': {
									actor ^= 1;
									body = `以降の効果は<span class="special">${battle.character[actor].name}</span>を発動者とする`;
								} break;
								default: {
									body = a[0];
								}
							}
							const p = document.createElement('p');
							if (pre !== null) {
								const span = document.createElement('span');
								span.innerText = hide_text(pre, this.text_hide);
								p.appendChild(span);
							}
							const span = document.createElement('span');
							span.innerHTML = hide_text(body, this.text_hide);
							p.appendChild(span);
							div.appendChild(p);
						});
					}
					// 表示更新
					if (version < 1) battle.update();
				}
				if (turn['status'] !== null && turn['status'] !== undefined) {
					battle.character[0].status.hp = turn['status'][0]['hp'];
					battle.character[0].status.mp = turn['status'][0]['mp'];
					battle.character[0].status.atk = turn['status'][0]['atk'];
					battle.character[0].status.tec = turn['status'][0]['tec'];
					battle.character[1].status.hp = turn['status'][1]['hp'];
					battle.character[1].status.mp = turn['status'][1]['mp'];
					battle.character[1].status.atk = turn['status'][1]['atk'];
					battle.character[1].status.tec = turn['status'][1]['tec'];
					battle.update();
				}
				battle.elem_log.appendChild(div);
				if (this.text_hide !== undefined) {
					this.text_hide = (battle.now - 8) / 128;
					console.log(this.text_hide);
				}
				return false;
			}
		}
		return false;
	}
}