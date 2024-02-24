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
			localStorage.setItem('timeline','[{"name":"現在位置","get":"{\'num\':20}"},{"name":"自分宛て","get":"{\'num\':20,\'from\':0}"}]');
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