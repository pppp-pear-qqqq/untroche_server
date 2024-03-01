function register() {
	const form = document.getElementById('register');
	let data = {
		password: form.querySelector('[name=password]').value,
		name: form.querySelector('[name=name]').value,
		acronym: form.querySelector('[name=acronym]').value,
		color: form.querySelector('[name=color]').value,
		fragment: [],
	};
	Array.prototype.forEach.call(form.querySelectorAll('#fragments select'), elem => {
		const op = elem.options[elem.selectedIndex];
		data['fragment'].push({ name: op.label, lore: op.innerHTML });
	});
	console.log(data);
	ajax.open({
		url: 'strings/register',
		ret: 'text',
		post: data,
		ok: ret => {
			Cookie.make('login_session', ret).path().max_age(60 * 60 * 24 * 7).set();
			reset_timeline();
			location.reload();
		}
	});
}

function login() {
	const form = document.getElementById('login');
	let data = {};
	data['eno'] = Number(form.querySelector('[name="eno"]').value);
	data['password'] = form.querySelector('[name="password"]').value;
	console.log(data);
	ajax.open({
		url: 'strings/login',
		ret: 'text',
		post: data,
		ok: ret => {
			Cookie.make('login_session', ret).path().max_age(60 * 60 * 24 * 7).set();
			location.reload();
		}
	});
}

function random_name() {
	let set;
	let min;
	let max;
	switch (Math.floor(Math.random() * 3)) {
		case 0: {
			set = [...'あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわゐゑをがぎぐげござじずぜぞだぢづでどばびぶべぼぱぴぷぺぽんゔぁぃぅぇぉゃゅょっ'];
			min = 2;
			max = 6;
		} break; 
		case 1: {
			set = [...'アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヰヱヲガギグゲゴザジズゼゾダヂヅデドバビブベボパピプペポンヴァィゥェォャュョッ'];
			min = 2;
			max = 6;
		} break;
		case 2: {
			set = [...'abcdefghijklmnopqrstuvwxyz'];
			min = 3;
			max = 12;
		} break;
	}
	let len = Math.floor(Math.random() * (max - min) + min);
	let name = '';
	for (let i = 0; i < len; ++i) {
		name += set[Math.floor(Math.random() * set.length)];
		if (i == 0)
			name = name.toUpperCase();
	}
	document.getElementById('name').value = name;
	document.getElementById('acronym').value = name.charAt(0);
}

function random_acronym() {
	set = [...'あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわゐゑをがぎぐげござじずぜぞだぢづでどばびぶべぼぱぴぷぺぽんゔぁぃぅぇぉゃゅょっアイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヰヱヲガギグゲゴザジズゼゾダヂヅデドバビブベボパピプペポンヴァィゥェォャュョッABCDEFGHIJKLMNOPQRSTUVWXYZ'];
	document.getElementById('acronym').value = set[Math.floor(Math.random() * set.length)];
}

function random_color() {;
	document.getElementById('color').value = `#${Math.floor(Math.random() * 0x1000000).toString(16).padStart(6,'0')}`;
}

function random_fragment() {
	const fragments = document.getElementById('fragments');
	Array.prototype.forEach.call(fragments.getElementsByTagName('label'), elem => {
		const select = elem.querySelector('select');
		const target = select.children[Math.floor(Math.random() * select.children.length)];
		target.selected = true;
		elem.querySelector('p').innerHTML = target.innerHTML;
	});
}

function toggle_form(target) {
	const back = document.getElementById('form');
	const login = document.getElementById('login');
	const register = document.getElementById('register');
	switch (target) {
		case 'login': {
			back.style.display = 'block';
			back.style.opacity = 1;
			register.classList.add('hide');
			login.classList.remove('hide');
		} break;
		case 'register': {
			back.style.display = 'block';
			back.style.opacity = 1;
			login.classList.add('hide');
			register.classList.remove('hide');
		} break;
		default: {
			setTimeout(() => back.style.display = 'none', 200);
			back.style.opacity = 0;
			login.classList.add('hide');
			register.classList.add('hide');
		}
	}
}

window.addEventListener('load', () => {
	const fragments = document.getElementById('fragments');
	const template = document.getElementById('template_fragment');
	for (let i = 0; i < 5; ++i) {
		const box = template.content.cloneNode(true);
		box.querySelector('select').onchange = event => {
			const e = event.currentTarget;
			e.nextElementSibling.innerHTML = e.options[e.selectedIndex].innerHTML;
		}
		fragments.appendChild(box);
	}
	document.getElementById('form').onclick = event => {
		if (event.target.id === 'form')
			toggle_form();
	};
});