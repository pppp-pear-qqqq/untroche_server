// 変数・関数の命名規則はrust準拠・及び標準との区別のためsnake_caseとする

// Ajaxインスタンスの初期化
// alertifyを要求する関係でajax.class.jsには置いていない
ajax.ok = ret => {
	alertify.success(ret);
}
ajax.err = ret => {
	console.error(ret);
	alertify.error(ret, 0);
}

var lastFocus;
window.addEventListener('focusin', () => lastFocus = document.activeElement);
/**
 * 選択されているテキスト位置に対して、preとpostで挟むように文字列を追加する
 * @param {string} pre 
 * @param {string} post 
 */
function add_decoration_tag(pre, post) {
	if (lastFocus !== undefined && lastFocus.tagName === 'TEXTAREA') {
		const start = lastFocus.selectionStart;
		const end = lastFocus.selectionEnd;
		lastFocus.value = lastFocus.value.substr(0, start) + pre + lastFocus.value.substr(start, end - start) + post + lastFocus.value.substr(end);
		lastFocus.focus();
		lastFocus.selectionStart = lastFocus.selectionEnd = end + pre.length + (start !== end) * post.length;
	}
}

function make_skillfomula(array) {
	let stack = [];
	let effect = [];
	array.forEach(e => {
		switch (e) {
			case '正': stack.push(`+${stack.pop()}`); break;
			case '負': stack.push(`-${stack.pop()}`); break;
			case '+': stack.push(`${stack.pop()} + ${stack.pop()}`); break;
			case '-': stack.push(`${stack.pop()} - ${stack.pop()}`); break;
			case '*': stack.push(`${stack.pop()} * ${stack.pop()}`); break;
			case '/': stack.push(`${stack.pop()} / ${stack.pop()}`); break;
			case '%': stack.push(`${stack.pop()} % ${stack.pop()}`); break;
			case '~': stack.push(`${stack.pop()} ~ ${stack.pop()}`); break;
			case '消耗':
			case '強命消耗':
			case '確率':
			case '攻撃':
			case '貫通攻撃':
			case '精神攻撃':
			case '回復':
			case '自傷':
			case '集中':
			case 'ATK変化':
			case 'TEC変化':
			case '移動':
			case '間合変更':
			case '逃走ライン': effect.push(`${e}(${stack.pop()})`); break;
			case '間合': effect.push(`${e}(${stack.pop()}, ${stack.pop()})`); break;
			case '中断':
			case '対象変更': effect.push(e); break;
			default: stack.push(e);
		}
	});
	return effect.join(', ');
}

function array_to_colorcode(arg) {
	return `${arg[0].toString(16).padStart(2,'0')}${arg[1].toString(16).padStart(2,'0')}${arg[2].toString(16).padStart(2, '0')}`
}

function reset_timeline() {
	localStorage.setItem('timeline','[{"name":"現在位置","get":"{\'num\':20}"},{"name":"自分宛て","get":"{\'num\':\'20\',\'to\':0,\'location\':\'*\'}"},{"name":"自分発言","get":"{\'num\':\'20\',\'from\':0,\'location\':\'*\'}"}]');
}

/**
 * タグをいい感じに置換した文字列を返す
 * ついでに特殊文字をエスケープする
 * @param {string} text 
 */
function replace_decoration_tag(text) {
	return text
		.replaceAll('&','&amp;')
		.replaceAll('"','&quot;')
		.replaceAll('\'','&#039;')
		.replaceAll('<','&lt;')
		.replaceAll('>','&gt;')
		.replace(/\r|\n|\r\n/g,'<br>')
		.replace(/\[(.+)\|(.*)\|\1\]/g, (_, p1, p2) => {
			switch (p1) {
				case 'b': case 'bold': return `<span class="bold">${p2}</span>`;
				case 'i': case 'italic': return `<span class="italic">${p2}</span>`;
				case 'u': case 'underline': return `<span class="underline">${p2}</span>`;
				case 's': case 'linethrough': return `<span class="linethrough">${p2}</span>`;
				case 'large': return `<span class="large">${p2}</span>`;
				case 'small': return `<span class="small">${p2}</span>`;
				case 'rainbow': return `<span class="rainbow">${p2}</span>`;
				default: return `{{${p1}|${p2}|${p1}}}`;
			}
		})
		.replaceAll('{{', '[')
		.replaceAll('}}', ']');
}

/**
 * 対象のコンテナにデータを変換しながら追加
 * @param {HTMLElement} container 追加先のコンテナ
 * @param {Array} data 元データ
 * @param {Function} f 変換用の関数、1行を受け取りHTMLElementを返す
 * @param {?Node} empty_elem データが空だった場合に追加する要素
 * @param {?boolean} reverse trueにするとコンテナにデータを追加する順番を逆順にする
 */
function load(container, data, f, empty_elem, reverse) {
	container.replaceChildren();
	if (data.length === 0 && empty_elem !== null && empty_elem !== undefined)
		container.appendChild(empty_elem);
	else data.forEach(i => {
		if (reverse)
			container.insertBefore(f(i), container.firstChild);
		else
			container.appendChild(f(i));
	});
}

/**
 * 文字列からHTMLElementを生成する
 * 兄弟関係にあるような複数の要素を生成しようとしたとき、最初の要素しか取得できない
 * @param {string} html 
 * @returns {?HTMLElement}
 */
function make_element(html) {
	const div = document.createElement('div');
	div.innerHTML = html;
	return div.firstElementChild;
}
