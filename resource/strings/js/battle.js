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
		this.start_status = new Status(raw);
		this.status = new Status(raw);
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
class Battle {
	constructor(raw) {
		this.rule = raw['rule'];
		this.version = raw['version'];
		this.auto = null;
		if (this.rule === "strings") {
			if(this.version < 1) {
				// 要素取得
				this.display = document.getElementById('play_battle');
				this.elem_log = this.display.querySelector('.log');
				const scroll = this.elem_log.parentNode;
				scroll.onclick = () => this.next(this);
				this.elem_range = this.display.querySelector('.range>.range');
				// 初期設定
				this.character = [new Character(raw['left'], this.display.querySelector('.data.left')), new Character(raw['right'], this.display.querySelector('.data.right'))];
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
			}
		}
	}
	update() {
		this.elem_range.style.width = `min(calc(24em * ${this.range} / ${this.escape_range} + 2em), 24em)`
		this.elem_range.innerText = this.range;
		this.character[0].update();
		this.character[1].update();
	}
	async close() {
		this.display.classList.add('hide');
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
		if (battle.rule === 'strings') {
			if (battle.version < 1) {
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
				// 表示位置の決定・行動者の表示
				switch (turn['owner']) {
					case 'strings': {
						div.classList.add('p_center');
					} break;
					case 'left': {
						actor = 0;
						div.classList.add('p_left');
						if (battle.log[battle.now] !== undefined) {
							const p = document.createElement('p');
							p.innerText = `${battle.character[actor].name}の行動`;
							div.appendChild(p);
						}
					} break;
					case 'right': {
						actor = 1;
						div.classList.add('p_right');
						if (battle.log[battle.now] !== undefined) {
							const p = document.createElement('p');
							p.innerText = `${battle.character[actor].name}の行動`;
							div.appendChild(p);
						}
					} break;
				}
				// 発言内容等
				if (turn['content'] !== null) {
					const p = document.createElement('p');
					p.innerHTML = turn['content'];
					div.appendChild(p);
				}
				// スキル名
				if (turn['skill'] !== null) {
					if (battle.version < 0.2) {
						const p = document.createElement('p');
						p.className = 'skill';
						p.innerHTML = turn['skill'];
						div.appendChild(p);
					} else {
						const p = document.createElement('p');
						p.className = 'skill';
						p.innerHTML = turn['skill'][0];
						if (turn['skill'][1] !== null)
							p.innerHTML += `<span class="tip">(${turn['skill'][1]})<span>`;
						div.appendChild(p);
					}
				}
				// スキル効果
				if (turn['action'] !== null) {
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
								pre = `${battle.character[actor^1].name}への攻撃`
								body = `<span class="minus">${value}</span>のダメージ`;
								battle.character[actor^1].status.hp -= value;
							} break;
							case '貫通攻撃': {
								pre = `${battle.character[actor^1].name}への攻撃`
								body = `<span class="minus">${value}</span>のダメージ`;
								battle.character[actor^1].status.hp -= value;
							} break;
							case '精神攻撃': {
								pre = `${battle.character[actor^1].name}への精神攻撃`
								body = `MPに<span class="special">${value}</span>のダメージ`;
								battle.character[actor^1].status.mp -= value;
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
								body = `以降の効果は<span class="special">${battle.character[actor]}</span>を発動者とする`;
							} break;
						}
						const p = document.createElement('p');
						if (pre !== null) {
							const span = document.createElement('span');
							span.innerText = pre;
							p.appendChild(span);
						}
						const span = document.createElement('span');
						span.innerHTML = body;
						p.appendChild(span);
						div.appendChild(p);
					});
					// 表示更新
					battle.update();
				}
				battle.elem_log.appendChild(div);
				return false;
			}
		}
		return false;
	}
}