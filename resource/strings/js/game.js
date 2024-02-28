var hold_fragment = null;
var desc_fragment = null;
var explore_text = [];
var explore_selectable = false;
var explore_ok = true;
var loading_profile_eno = null;
var select_tab_timeline = null;
var battle = null;
var timeline_tabs = JSON.parse(localStorage.getItem('timeline'));

function logout() {
	Cookie.delete('login_session');
	location.reload();
}

// ページ遷移系
function view_window(e) {
	Array.prototype.forEach.call(e.parentNode.children, elem => elem.classList.remove('select'));
	e.classList.add('select');
	const target = e.dataset.target.split(',');
	Array.prototype.forEach.call(document.getElementById('main').children, elem => {
		if (target.includes(elem.id))
			elem.classList.remove('hide');
		else
			elem.classList.add('hide');
	});
}
function to_chat(to, lock, content) {
	document.querySelector('#chat>.talk>.to>input').value = to;
	document.querySelector('#chat>.talk>.lock>input').checked = lock;
	if (content !== undefined && content !== null)
		document.querySelector('#chat>.talk>.word').value += content;
	view_window(document.querySelector('#footer>[data-target="location,chat"]'));
}
function to_profile() {
	Array.prototype.forEach.call(document.getElementById('footer').children, elem => elem.classList.remove('select'));
	Array.prototype.forEach.call(document.getElementById('main').children, elem => {
		if (elem.id === 'profile')
			elem.classList.remove('hide');
		else
			elem.classList.add('hide');
	});
}

// ローディング系
/**
 * キャラクターリストを更新
 * @param {HTMLElement} container 
 * @param {Number} num 
 * @param {?Number} start 
 * @param {?string} location 
 */
function load_characters(container, num, start, location) {
	ajax.open({
		url: 'strings/get_characters',
		ret: 'json',
		get: {num, start, location},
		ok: ret => {
			const template = document.getElementById('template_character');
			load(container, ret, i => {
				const e = template.content.cloneNode(true).querySelector('.character');
				e.style.borderColor = `#${array_to_colorcode(i['color'])}`;
				e.onclick = event => {
					const e = event.target;
					if (!e.classList.contains('to') && !e.classList.contains('battle')) {
						if (loading_profile_eno === Number(i['eno'])) {
							document.getElementById('profile').classList.add('hide');
							loading_profile_eno = null;
						} else
							load_profile(i['eno']);
					}
				};
				e.querySelector('.eno').innerText = `Eno.${i['eno']}`;
				e.querySelector('.acronym').innerText = i['acronym'];
				e.querySelector('.name').innerText = i['name'];
				e.querySelector('.word').innerText = i['comment'];
				e.querySelector('.location').innerText = i['location'];
				if (i['eno'] === eno)
					e.querySelector('.footer').remove();
				else {
					e.querySelector('.to').onclick = () => to_chat(i['eno'], false);
					e.querySelector('.battle.win').onclick = () => send_battle(i['eno'], 1);
					e.querySelector('.battle.lose').onclick = () => send_battle(i['eno'], 0);
				}
				return e;
			}, make_element('<div class="character"><p class="word">キャラクターが存在しません</p></div>'));
		}
	});
}
/**
 * タイムラインを更新
 * @param {HTMLElement} container 
 * @param {Number} num 最大取得件数
 * @param {?Number} start 取得開始位置
 * @param {?Number} from 発言者
 * @param {?Number} to 対象者
 * @param {?string} location 取得ロケーション
 * @param {?string} word 文字列検索
 */
function load_timeline(container, num, start, from, to, location, word) {
	ajax.open({
		url: 'strings/get_chat',
		ret: 'json',
		get: {num: num, start: start, from: from, to: to, location: location, word: word},
		ok: ret => {
			const template = document.getElementById('template_talk');
			load(container, ret, i => {
				const e = template.content.cloneNode(true);
				if (i['to'] !== null) {
					const to = e.querySelector('.to');
					to.innerText = `>> Eno.${i['to']}`
					to.classList.remove('hide');
				}
				const location = e.querySelector('.location');
				if (i['location'] === null)
					location.innerHTML = '<img src="pic/lock_fill.svg" width="21" height="21">';
				else
					location.innerText = i['location'];
				e.querySelector('.name').innerText = i['name'];
				e.querySelector('.eno').innerText = `Eno.${i['from']}`;
				e.querySelector('.id').innerText = `id:${i['id']}`;
				e.querySelector('.timestamp').innerText = i['timestamp'];
				e.querySelector('.word').innerHTML = i['word'];
				e.querySelector('.talk').style.borderColor = `#${array_to_colorcode(i['color'])}`;
				e.querySelector('.reply').onclick = () => to_chat(i['from'], false, `>>${i['id']}`);
				return e;
			}, make_element('<div class="talk"><p class="word">発言がありません</p></div>'), true);
		}
	});
}
function load_fragments(main_container, trash_container) {
	ajax.open({
		url: 'strings/get_fragments',
		ret: 'json',
		get: {},
		ok: ret => {
			const template = document.getElementById('template_fragment');
			main_container.replaceChildren();
			for (let i = 1; i <= 30; ++i) {
				const f = ret.find(f => f['slot'] == i);
				const e = template.content.cloneNode(true).querySelector('div');
				if (f !== undefined) {
					e.draggable = true;
					e.dataset.slot = i;
					e.dataset.category = f['category'];
					e.dataset.lore = f['lore'];
					e.querySelector('.name').innerText = f['name'];
					const status = e.querySelector('.status');
					status.dataset.hp = (f['hp']<0?'':'+')+f['hp'];
					status.dataset.mp = (f['mp']<0?'':'+')+f['mp'];
					status.dataset.atk = (f['atk']<0?'':'+')+f['atk'];
					status.dataset.tec = (f['tec']<0?'':'+')+f['tec'];
					const s = e.querySelector('.skill');
					if (f['skill']) {
						s.dataset.name = f['skill']['name'];
						s.dataset.defaultname = f['skill']['default_name'];
						s.innerText = (s.dataset.name !== '') ? s.dataset.name : s.dataset.defaultname;
						s.dataset.word = f['skill']['word'];
						s.dataset.lore = f['skill']['lore'];
						s.dataset.timing = f['skill']['timing'];
						s.dataset.effect = f['skill']['effect'];
					} else {
						s.classList.add('none');
					}
					e.ondragstart = event => {
						hold_fragment = event.currentTarget;
						hold_fragment.classList.add('hold');
					};
					e.ondragend = () => {
						hold_fragment.classList.remove('hold');
						hold_fragment = null;
						update_status(main_container);
					};
					e.onclick = open_desc;
				} else {
					e.classList.add('none');
				}
				e.ondragenter = trade_fragment;
				main_container.appendChild(e);
			}
			trash_container.replaceChildren();
			for (let i = 1; i <= 30; ++i) {
				const e = template.content.cloneNode(true).querySelector('div');
				e.classList.add('none');
				e.ondragenter = trade_fragment;
				trash_container.appendChild(e);
			}
		}
	})
}
function load_profile(eno) {
	ajax.open({
		url: 'strings/get_profile',
		ret: 'json',
		get: {eno: eno},
		ok: ret => {
			const template = document.getElementById('template_profile_fragment');
			const e = document.querySelector('#profile>div');
			e.querySelector('.eno').innerText = `Eno.${ret['eno']}`;
			e.querySelector('.fullname').innerText = (ret['fullname']!=='')?ret['fullname']:'────────────────────────';
			load(e.querySelector('.fragments'), ret['fragments'], i => {
				const e = template.content.cloneNode(true);
				e.querySelector('.name').innerText = i['name'];
				e.querySelector('.category').innerText = i['category'];
				e.querySelector('.lore').innerHTML = i['lore'];
				return e;
			});
			const acronym = e.querySelector('.acronym');
			const color = e.querySelector('.color');
			const name = e.querySelector('.name');
			const comment = e.querySelector('.comment');
			const profile = e.querySelector('.profile');
			const memo = e.querySelector('.memo');
			color.value = `#${array_to_colorcode(ret['color'])}`;
			comment.value = ret['comment'];
			profile.querySelector('p').innerHTML = ret['profile'];
			memo.querySelector('p').innerHTML = ret['memo'];
			if(ret['edit_mode']) {
				const input = document.createElement('input');
				input.type = 'text';
				input.value = ret['acronym'];
				input.onchange = event => update_profile('acronym', event.currentTarget);
				input.onfocus = event => {
					const e = event.currentTarget;
					e.dataset.prev = e.value;
				};
				acronym.replaceChildren(input);
				color.disabled = false;
				name.querySelector('input').value = ret['name'];
				name.classList.remove('hide');
				comment.disabled = false;
				profile.dataset.editable = true;
				profile.querySelector('textarea').value = ret['raw_profile'];
				memo.dataset.editable = true;
				memo.querySelector('textarea').value = ret['raw_memo'];
			} else {
				acronym.innerText = ret['acronym'];
				color.disabled = true;
				name.classList.add('hide');
				comment.disabled = true;
				profile.dataset.editable = false;
				memo.dataset.editable = false;
			}
			document.getElementById('profile').classList.remove('hide');
			loading_profile_eno = Number(ret['eno'])
		}
	})
}
function load_battle_reserve(container) {
	ajax.open({
		url: 'strings/get_battle_reserve',
		ret: 'json',
		get: {},
		ok: ret => {
			const template = document.getElementById('template_battle_reserve');
			load(container, ret, i => {
				const e = template.content.cloneNode(true);
				if (i['from'][0] === eno) {
					e.querySelector('.message').innerHTML = `Eno.${i['to'][0]} ${i['to'][1]} へ戦闘を挑んでいます。`;
					e.querySelector('.option').innerHTML = `<a onclick="cancel_battle(${i['to'][0]});this.closest('.item').remove()">取り下げる</a>`;
				} else {
					e.querySelector('.message').innerHTML = `<a onclick="load_profile(${i['from'][0]});to_profile()">Eno.${i['from'][0]} ${i['from'][1]}</a> に戦闘を挑まれています。`;
					e.querySelector('.option').innerHTML = `<a onclick="to_chat(${i['from'][0]},true)">話し合う</a><a onclick="receive_battle(${i['from'][0]},1);this.closest('.item').remove()">勝つつもりで挑む</a><a onclick="receive_battle(${i['from'][0]},0);this.closest('.item').remove()">負けるつもりで挑む</a><a onclick="receive_battle(${i['from'][0]},-1);this.closest('.item').remove()">逃げる</a>`;
				}
				return e;
			});
		}
	})
}
function load_battle_logs(container, eno) {
	ajax.open({
		url: 'strings/get_battle_logs',
		ret: 'json',
		get: {eno: eno},
		ok: ret => {
			const template = document.getElementById('template_battle_log');
			load(container, ret, i => {
				const e = template.content.cloneNode(true).querySelector('div');
				const left = e.querySelector('.left');
				const right = e.querySelector('.right');
				e.dataset.id = i['id'];
				left.querySelector('.eno').innerText = `Eno.${i['left']['eno']}`;
				left.querySelector('.name').innerText = i['left']['name'];
				right.querySelector('.eno').innerText = `Eno.${i['right']['eno']}`;
				right.querySelector('.name').innerText = i['right']['name'];
				const left_color = `#${array_to_colorcode(i['left']['color'])}`;
				const right_color = `#${array_to_colorcode(i['right']['color'])}`;
				e.style.borderColor = `${left_color} ${right_color} ${right_color} ${left_color}`;
				switch (i['result']) {
					case 'left': left.classList.add('win'); right.classList.add('lose'); break;
					case 'right': right.classList.add('win'); left.classList.add('lose'); break;
					case 'draw': e.classList.add('draw'); break;
					case 'escape': e.classList.add('escape'); break;
				}
				e.onclick = event => {
					let e = event.target;
					if (e.classList.contains('eno') || e.classList.contains('name'))
						e = e.parentNode;
					if (e.classList.contains('left')) {
						// 左方キャラクターのプロフィール
						load_profile(i['left']['eno']);
						to_profile();
					} else if (e.classList.contains('right')) {
						// 右方キャラクターのプロフィール
						load_profile(i['right']['eno']);
						to_profile();
					} else {
						// ログ再生
						ajax.open({
							url: 'strings/get_battle_log',
							ret: 'json',
							get: {id: i['id']},
							ok: ret => {
								battle = new Battle(ret);
							}
						})
					}
				}
				return e;
			}, make_element('<div>まだ一件も戦闘していません</div>'));
		}
	})
}
function load_location(location) {
	ajax.open({
		url: 'strings/get_location',
		ret: 'json',
		get: {location: location},
		ok: ret => {
			const e = document.querySelector('#location .location');
			e.querySelector('.name').innerText = ret['name'];
			e.querySelector('.lore').innerHTML = ret['lore'];
		}
	})
}

// 探索
function explore() {
	if (explore_ok) {
		if (explore_text.length > 0) {
			// exploreの最初の行を取得
			const log = document.querySelector('#explore .log');
			let next = explore_text.shift();
			if (next.includes('$input-nullable')) {
				next = next.replaceAll('$input-nullable', '<input type="text" placeholder="16文字以下"><a onclick="next(this.previousElementSibling.value,this)">決定</a>');
				explore_selectable = true;
			}
			if (next.includes('$input')) {
				next = next.replaceAll('$input', '<input type="text" placeholder="16文字以下"><a onclick="next(this.previousElementSibling.value,this,true)">決定</a>');
				explore_selectable = true;
			}
			if (next.includes('$')) {
				next = next.replaceAll(/\$\((.+?),(.+?)\)/g, '<a onclick="next(\'$2\',this)">$1</a>');
				explore_selectable = true;
			}
			const p = document.createElement('p');
			p.innerHTML = next.trim();
			log.appendChild(p);
		} else if (!explore_selectable) {
			next();
		}
	}
}
function next(input,this_elem,strict) {
	if (strict && (input === undefined || input === null ||input === '')) {
		alertify.error('入力されていません');
		return;
	}
	explore_ok = false;
	// 選択肢を無効にする
	const log = document.querySelector('#explore .log');
	log.querySelectorAll('a').forEach(elem => {
		const rep = document.createElement('span');
		rep.innerHTML = elem.innerHTML;
		if (elem === this_elem) rep.classList.add('underline');
		elem.replaceWith(rep);
	});
	log.querySelectorAll('input').forEach(elem => {
		const rep = document.createElement('span');
		rep.innerHTML = elem.value !== "" ? elem.value : "　　　　";
		rep.classList.add('underline');
		elem.nextElementSibling.remove();
		elem.replaceWith(rep);
	});
	explore_selectable = false;
	// 取得
	if (input === '') input = null;
	return ajax.open({
		url: 'strings/next',
		ret: 'text',
		post: {value: input},
		ok: (ret) => {
			// テキスト全文が来る
			explore_text.push(...ret.split('\n'));
			explore_ok = true;
			load_location();
			load_fragments(document.querySelector('#fragment .container'), document.querySelector('#fragment .container.trash'));
			explore();
		}
	})
}

// チャット
function talk() {
	const form = document.querySelector('#chat>.talk');
	const name = form.querySelector('[name=name]').value;
	const word = form.querySelector('[name=word]');
	const location = form.querySelector('[name=location]').checked;
	const to = form.querySelector('[name=to]').value;
	if (word.value !== '') {
		ajax.open({
			url: 'strings/send_chat',
			post: {name: name, word: word.value, location: location, to: (to!=="")?Number(to):null},
			ok: () => {
				alertify.success("発言しました");
				word.value = '';
				const params = JSON.parse(select_tab_timeline.dataset.get);
				load_timeline(document.querySelector('#chat .log'), params['num'], params['start'], params['from'], params['to'], params['location'], params['word']);
			}
		});
	} else alertify.error('発言内容がありません');
}
function preview_open() {
	const form = document.querySelector('#chat>.talk');
	const name = form.querySelector('[name=name]').value;
	const word = form.querySelector('[name=word]').value;
	const location= form.querySelector('[name=location]').checked;
	const to = form.querySelector('[name=to]').value;
	const e = document.querySelector('#chat>.preview');
	const e_to = e.querySelector('.to');
	if (to === '') {
		e_to.classList.add('hide');
	} else {
		e_to.innerText = `>> Eno.${to}`;
		e_to.classList.remove('hide');
	}
	const e_location = e.querySelector('.location');
	if (location)
		location.innerHTML = '<img src="pic/lock_fill.svg" width="21" height="21">';
	else
		location.innerText = '現在地';
	e.querySelector('.name').innerText = name;
	e.querySelector('.eno').innerText = `Eno.${eno}`;
	e.querySelector('.word').innerHTML = replace_decoration_tag(word);
	e.querySelector('.talk').style.borderColor = form.style.borderColor;
	e.classList.remove('hide');
}
function preview_close() {
	document.querySelector('#chat>.preview').classList.add('hide');
}
function search_timeline() {
	const form = document.querySelector('#chat>.main>.search');
	// 条件を設定してチャットを取得する
	let params = { num: 20 };
	const from = form.querySelector('[name=from]').value;
	const to = form.querySelector('[name=to]').value;
	const location = form.querySelector('[name=location]').value;
	const word = form.querySelector('[name=word]').value;
	if (from !== "") params['from'] = from;
	if (to !== "") params['to'] = to;
	if (location !== "") params['location'] = location;
	if (word !== "") params['word'] = word;
	// 条件を取得してチャットを取得し、反映する
	const timeline = document.querySelector('#chat .log');
	load_timeline(timeline, params['num'], null, params['from'], params['to'], params['location'], params['word']);
	form.classList.add('hide');
	timeline.classList.remove('hide');
	// 取得条件を反映したタブを作成する
	const p = document.createElement('p');
	p.className = 'select onetime';
	p.innerText = '検索結果';
	p.dataset.get = JSON.stringify(params);
	p.onclick = select_chat_tab;
	// 作ったタブを現在選択中のタブ（多分検索タブ）の前に追加
	select_tab_timeline.parentNode.insertBefore(p, select_tab_timeline);
	// 選択中のノードを変える
	select_tab_timeline.classList.remove('select');
	select_tab_timeline = p;
}
/**
 * タブを選択したときの動作
 * @param {Event} event 
 */
function select_chat_tab(event) {
	const elem = event.currentTarget;
	// テンプレートタブ
	if (elem === select_tab_timeline) {
		// 選択状態だった
		elem.onclick = null;
		// 中身をInputElementに置換する
		const input = document.createElement('input');
		input.value = elem.innerText;
		// 入力が完了したとき、値を保存してInputElementからテキストに戻す
		input.onblur = () => {
			if (input.value === '') {
				const index = [].slice.call(elem.parentNode.children).indexOf(elem);
				elem.remove();
				timeline_tabs.splice(index, 1);
			} else if (elem.classList.contains('onetime')) {
				elem.classList.remove('onetime');
				timeline_tabs.push({ name: input.value, get: elem.dataset.get });
			} else {
				const index = [].slice.call(elem.parentNode.children).indexOf(elem);
				timeline_tabs[index]['name'] = input.value;
			}
			localStorage.setItem('timeline', JSON.stringify(timeline_tabs));
			elem.firstChild.replaceWith(input.value);
			elem.onclick = select_chat_tab;
		};
		elem.firstChild.replaceWith(input);
		input.focus();
	} else {
		// 選択状態でなければ選択対象を切り替える
		elem.classList.add('select');
		if (select_tab_timeline.classList.contains('onetime'))
			select_tab_timeline.remove();
		else {
			if (select_tab_timeline.classList.contains('search')) {
				document.querySelector('#chat>.main>.search').classList.add('hide');
				document.querySelector('#chat .log').classList.remove('hide');
			}
			select_tab_timeline.classList.remove('select');
		}
		select_tab_timeline = elem;
		// 条件を取得してチャットを取得し、反映する
		const params = JSON.parse(select_tab_timeline.dataset.get);
		load_timeline(document.querySelector('#chat .log'), params['num'], params['start'], params['from'], params['to'], params['location'], params['word']);
	}
}

// フラグメント
function trade_fragment(event) {
	const target = event.currentTarget;
	if (hold_fragment !== null && target !== hold_fragment) {
		const next = target.nextElementSibling;
		const parent = hold_fragment.parentNode;
		hold_fragment.classList.add('changed');
		if (!target.classList.contains('none'))
			target.classList.add('changed');
		if(hold_fragment === next) {
			parent.insertBefore(hold_fragment, target);
		} else {
			parent.insertBefore(target, hold_fragment);
			parent.insertBefore(hold_fragment, next);
		}
	}
}
function open_desc(event) {
	const e = event.currentTarget;
	const desc = document.querySelector('#fragment>.desc');
	if (desc_fragment !== e) {
		desc_fragment = e;
		desc.querySelector('.name').innerText = e.querySelector('.name').innerText;
		desc.querySelector('.category').innerText = e.dataset.category;
		desc.querySelector('.lore').innerHTML = e.dataset.lore
		const status = e.querySelector('.status');
		desc.querySelector('.status').innerText = `${'HP' + status.dataset.hp}, ${'MP' + status.dataset.mp}, ${'ATK' + status.dataset.atk}, ${'TEC' + status.dataset.tec}`;
		const skill = e.querySelector('.skill');
		const desc_skill = desc.querySelector('.skill');
		if (skill.classList.contains('none')) {
			desc_skill.classList.add('hide');
		} else {
			desc_skill.classList.remove('hide');
			const desc_skill_name = desc_skill.querySelector('.name');
			desc_skill_name.value = skill.dataset.name;
			desc_skill_name.placeholder = skill.dataset.defaultname;
			desc_skill.querySelector('.word').value = skill.dataset.word;
			desc_skill.querySelector('.lore').innerHTML = skill.dataset.lore;
			desc_skill.querySelector('.timing').innerText = skill.dataset.timing;
			desc_skill.querySelector('.effect').innerText = skill.dataset.effect;
		}
		desc.classList.add('on');
	} else {
		desc_fragment = null;
		desc.classList.remove('on');
	}
}
function update_status(list) {
	let hp = 0;
	let mp = 0;
	let atk = 0;
	let tec = 0;
	for (let i = 0; i < 10; ++i) {
		if (list.length <= i) break;
		if (!list.children[i].classList.contains('none')) {
			const status = list.children[i].querySelector('.status');
			hp += Number(status.dataset.hp);
			mp += Number(status.dataset.mp);
			atk += Number(status.dataset.atk);
			tec += Number(status.dataset.tec);
		}
	}
	const status = document.querySelector('#fragment>.footer>.status');
	status.dataset.hp = hp;
	status.dataset.mp = mp;
	status.dataset.atk = atk;
	status.dataset.tec = tec;
}

// プロフィール
/**
 * 
 * @param {'acronym'|'color'|'name'|'profile'|'memo'} type 
 * @param {HTMLInputElement} elem 
 */
function update_profile(type, elem) {
	ajax.open({
		url: 'strings/update_profile',
		ret: 'text',
		post: { data_type: type, value: elem.value },
		ok: ret => {
			alertify.success('更新しました');
			switch (type) {
				case 'profile': document.querySelector('#profile .profile>p').innerHTML = ret; break;
				case 'memo': document.querySelector('#profile .memo>p').innerHTML = ret; break;
				case 'color': document.querySelector('#chat>.talk').style.borderColor = elem.value;
			}
		},
		err: ret => {
			alertify.error(ret, 0);
			switch (type) {
				case 'acronym':
				case 'color':
				case 'name': elem.value = elem.dataset.prev;
			}
		}
	})
}

// 戦闘ログ
function send_battle(to, plan) {
	ajax.open({
		url: 'strings/send_battle',
		ret: 'text',
		post: {to: to, plan: plan},
		ok: ret => {
			if (plan != 0) alertify.success('Eno.' + to + ' ' + ret + 'に勝つつもりで戦闘を挑みました');
			else alertify.success('Eno.' + to + ' ' + ret + 'に負けるつもりで戦闘を挑みました');
		}
	})
}
function receive_battle(from, plan) {
	ajax.open({
		url: 'strings/receive_battle',
		ret: 'json',
		post: {from: from, plan: plan},
		ok: ret => {
			if (ret['result'] === 'omission') {
				alertify.success(ret['content']);
			} else {
				battle = new Battle(ret[0]);
				if (ret[0]['result'] === 'right') {
					alertify.success(`フラグメント『${ret[1]}』を獲得しました`);
				} else if (ret[0]['result'] === 'left') {
					alertify.message(`フラグメント『${ret[1]}』を喪失しました`);
				}
				load_battle_logs(document.querySelector('#battle>.log'), eno);
				load_fragments(document.querySelector('#fragment .container'), document.querySelector('#fragment .container.trash'));
			}
		}
	})
}
function cancel_battle(to) {
	ajax.open({
		url: 'strings/cancel_battle',
		ret: 'text',
		post: {to: to},
		ok: ret => {
			alertify.success(ret);
		}
	})
}

window.addEventListener('load', async () => {
	// ========================
	// 機能の設定
	// ========================
	document.querySelectorAll('#footer>.tab').forEach(elem => {
		elem.onclick = event => view_window(event.currentTarget);
	});
	// タイムラインタブを反映
	{
		const tab_timeline = document.querySelector('#chat>.tabs');
		if (timeline_tabs !== null)
			load(tab_timeline, timeline_tabs, tab => {
				const p = document.createElement('p');
				p.dataset.get = tab['get'].replaceAll("'", '"');
				p.innerText = tab['name'];
				p.onclick = select_chat_tab;
				if (select_tab_timeline === null) {
					p.classList.add('select');
					select_tab_timeline = p;
				}
				return p;
			});
		const p = document.createElement('p');
		p.className = 'search';
		p.innerText = '検索';
		p.onclick = event => {
			const e = event.currentTarget;
			// 未選択なら
			if (e !== select_tab_timeline) {
				// これを選択する
				if (select_tab_timeline.classList.contains('onetime'))
					select_tab_timeline.remove();
				else
					select_tab_timeline.classList.remove('select');
				e.classList.add('select');
				select_tab_timeline = e;
				// 検索フォームを開く
				document.querySelector('#chat .log').classList.add('hide');
				document.querySelector('#chat>.main>.search').classList.remove('hide');
			}
		}
		tab_timeline.appendChild(p);
	}

	// 発言窓の自動整形関数
	// document.querySelector('#chat>.talk>textarea.word').oninput = event => {
	// 	const e = event.currentTarget;
	// 	e.style.height = 0;
	// 	e.style.height = `${e.scrollHeight}px`;
	// };

	// フラグメント選択更新機能
	document.querySelectorAll('#fragment>.desc>.skill>input').forEach(elem => {
		elem.onchange = event => {
			if (desc_fragment !== null) {
				const e = event.currentTarget;
				const target = desc_fragment.querySelector('.skill');
				target.parentNode.classList.add('changed');
				if (e.classList.contains('name')) {
					target.dataset.name = e.value;
					if (e.value === '') {
						target.innerText = e.placeholder;
					} else {
						target.innerText = e.value;
					}
				} else if (e.classList.contains('word')) {
					target.dataset.word = e.value;
				}
			}
		}
	});
	{
		const trash_tab = document.querySelector('#fragment>.footer>.trash');
		trash_tab.ondragover = event => event.preventDefault();
		trash_tab.ondrop = event => {
			event.preventDefault();
			if (hold_fragment !== null) {
				const container = [
					document.querySelector('#fragment>.container'),
					document.querySelector('#fragment>.container.trash'),
				];
				const t = event.currentTarget.classList.contains('open');
				const trade = container[t?0:1].querySelector('.fragment.none');
				if (trade !== null) {
					const next = trade.nextElementSibling;
					container[t?1:0].insertBefore(trade, hold_fragment);
					container[t?0:1].insertBefore(hold_fragment, next);
				} else alertify.error('これ以上移動させられません');
				hold_fragment.classList.add('changed');
			}
		}
		trash_tab.onclick = event => {
			const e = event.currentTarget;
			const container = [
				document.querySelector('#fragment>.container'),
				document.querySelector('#fragment>.container.trash'),
			];
			e.classList.toggle('open');
			t = e.classList.contains('open');
			container[t?0:1].classList.add('hide');
			container[t?1:0].classList.remove('hide');
		}
	}
	document.querySelector('#fragment>.footer>.update').onclick = () => {
		let changed = [];
		let trash = [];
		const fragments = document.querySelector('#fragment');
		const main_container = fragments.querySelector('.container');
		const trash_container = fragments.querySelector('.container.trash');
		main_container.querySelectorAll('.changed').forEach(elem => {
			const skill = elem.querySelector('.skill');
			changed.push({
				prev: Number(elem.dataset.slot),
				next: [].slice.call(main_container.children).indexOf(elem) + 1,
				skill_name: skill.dataset.name,
				skill_word: skill.dataset.word,
			});
		});
		trash_container.querySelectorAll('.changed').forEach(elem => trash.push(Number(elem.dataset.slot)));
		ajax.open({
			url: 'strings/update_fragments',
			ret: 'text',
			post: {change: changed, trash: trash},
			ok: ret => {
				alertify.success(ret);
				load_fragments(main_container, trash_container);
			}
		})
	}

	// プロフィール更新機能
	document.querySelectorAll('#profile>div>.text').forEach(elem => {
		elem.querySelector('p').onclick = event => {
			const e = event.currentTarget;
			if (e.parentNode.dataset.editable==="true") {
				e.classList.add('hide');
				e.nextElementSibling.classList.remove('hide');
				e.nextElementSibling.focus();
			}
		};
		const textarea = elem.querySelector('textarea');
		textarea.onblur = event => {
			const e = event.currentTarget;
			e.classList.add('hide');
			e.previousElementSibling.classList.remove('hide');
		};
		if (elem.classList.contains('profile'))
			textarea.onchange = event => update_profile('profile', event.currentTarget);
		else if (elem.classList.contains('memo'))
			textarea.onchange = event => update_profile('memo', event.currentTarget);
	});
	document.querySelector('#profile .comment').onchange = event => update_profile('comment', event.currentTarget);
	document.querySelector('#profile .color').onchange = event => update_profile('color', event.currentTarget);
	document.querySelector('#profile>div>.name>input').onchange = event => update_profile('name', event.currentTarget);
	document.querySelectorAll('#profile input').forEach(elem => {
		elem.addEventListener('focus', event => {
			const e = event.currentTarget;
			e.dataset.prev = e.value;
		});
	});
	document.querySelector('#other label>input').onchange = event => update_profile('webhook', event.currentTarget);

	// リスト更新
	document.querySelector('#location h4>.reload').onclick = async () => load_characters(document.querySelector('#location .characters'), 1000);
	document.querySelector('#other h4>.reload').onclick = async () => load_characters(document.querySelector('#other .characters'), 1000, null, '*');
	document.querySelector('#battle h4>.reload').onclick = () => load_battle_logs(document.querySelector('#battle .log'), eno);

	// 自分のプロフィール
	document.querySelector('#other .my_profile').onclick = () => load_profile(eno);

	// ========================
	// 要素の追加
	// ========================
	// シーンをひとつ更新
	explore();

	// 条件を取得してチャットを取得し、反映する
	if (select_tab_timeline !== null) {
		const params = JSON.parse(select_tab_timeline.dataset.get);
		load_timeline(document.querySelector('#chat .log'), params['num'], params['start'], params['from'], params['to'], params['location'], params['word']);
	}

	// キャラクターリストを取得・反映
	load_characters(document.querySelector('#location .characters'), 1000);
	load_characters(document.querySelector('#other .characters'), 1000, null, '*');

	// フラグメント
	load_fragments(document.querySelector('#fragment .container'), document.querySelector('#fragment .container.trash'));
	setTimeout(update_status,1000,document.querySelector('#fragment .container'));

	// 戦闘
	load_battle_reserve(document.querySelector('#battle .reserve'));
	load_battle_logs(document.querySelector('#battle .log'), eno);
});