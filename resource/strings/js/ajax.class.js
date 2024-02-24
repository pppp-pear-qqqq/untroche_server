class Ajax {
	#ok;
	#err;
	/**
	 * @param {Function} func 成功時のデフォルトコールバック
	 */
	set ok(func) { this.#ok = func; }
	/**
	 * @param {Function} func  失敗時のデフォルトコールバック
	 */
	set err(func) { this.#err = func; }
	/**
	 * @param {string} url 対象となるファイルのURL
	 * @param {string} ret 返り値として受け取るデータの種類
	 * @param {{[key:string]:any}} get GETメソッドで渡す引数
	 * @param {Object} post POSTメソッドで渡す引数
	 * @param {?Function} ok 成功時に実行されるコールバック
	 * @param {?Function} err 失敗時に実行されるコールバック
	 */
	async open({ url, ret = 'text', get, post, ok, err }) {
		let callback;
		if (get !== undefined) {
			Object.keys(get).forEach(key => {
				if (get[key] === undefined || get[key] === null)
					delete get[key];
			});
			url += '?' + new URLSearchParams(get).toString();
		}
		if (post !== undefined) {
			Object.keys(post).forEach(key => {
				if (post[key] === undefined || post[key] === null)
					delete post[key];
			});
		}
		return fetch(url, post !== undefined ? {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
			},
			body: JSON.stringify(post)
		} : undefined).then(response => {
			if (response.ok) {
				callback = ok !== undefined ? ok : this.#ok;
				switch (ret) {
					case 'arrayBuffer': return response.arrayBuffer();
					case 'blob': return response.blob();
					case 'formData': return response.formData();
					case 'json': return response.json();
					case 'text': return response.text();
				}
			} else {
				callback = err !== undefined ? err : this.#err;
				return response.text();
			}
		}).then(ret => {
			if (callback !== undefined) return callback(ret);
		});
	}
}
var ajax = new Ajax();