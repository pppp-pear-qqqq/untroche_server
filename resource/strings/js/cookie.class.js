class Cookie {
	#str;

	constructor(name, value) {
		this.#str = name + '=' + value;
	}

	/**
	 * Cookie文字列を作成する
	 * valueの値を指定しなかった場合、空の文字列が入る
	 * @param {string} name 
	 * @param {string?} value 
	 * @returns Cookie
	 */
	static make(name, value = '') {
		return new Cookie(name, value);
	};

	/**
	 * 有効パスを指定する
	 * 指定しなかった場合"/"が入る
	 * @param {string} path 
	 * @returns 
	 */
	path(path = '/') {
		this.#str += ';Path=' + path;
		return this;
	}

	/**
	 * 有効期限を秒数で指定する
	 * 指定しなかった場合0が入る
	 * @param {number} max_age 
	 */
	max_age(max_age = 0) {
		this.#str += ';Max-Age=' + max_age;
		return this;
	}

	/**
	 * Cookieを登録する
	 */
	set() {
		document.cookie = this.#str;
	}

	/**
	 * Cookieを削除する
	 * @param {string} name 
	 */
	static delete(name) {
		new Cookie(name, '').max_age().set();
	}

	/**
	 * Cookieの値を読み込む
	 * @param {string} name 
	 */
	static get(name) {
		return document.cookie.split("; ")
			.find((row) => row.startsWith(name))
			.split("=")[1];
	}
}

