function getBoxBlocksImports(wasmMemory, wasmExports) {
    // Index 0 is reserved as the null/invalid handle.
    const objects = [null];

    function store(obj) {
        if (obj == null) return 0;
        objects.push(obj);
        return objects.length - 1;
    }

    function get(idx) {
        return objects[idx];
    }

    function str(ptr, len) {
        return new TextDecoder().decode(new Uint8Array(wasmMemory.buffer, ptr, len));
    }

    function writeStr(s) {
        const encoded = new TextEncoder().encode(s + '\0');
        const ptr = wasmExports.alloc(encoded.length);
        new Uint8Array(wasmMemory.buffer).set(encoded, ptr);
        return ptr;
    }

    return {
        box_blocks: {
            external_get_our_username: () => {
                const username = new URL(window.location.href).pathname.split("/")[1];
                return writeStr(username);
            },
            external_format_time: (time) => {
                const date = new Date(time * 1000);
                return writeString(date.toLocaleString());
            },
            external_start_fetch_display_name: (usernamePtr, len, id) => {
                const username = str(usernamePtr, len);
                fetch(`/api/user/display-name?username=${encodeURIComponent(username)}`)
                    .then(res => res.json())
                    .then(data => {
                        const encoded = new TextEncoder().encode(data.displayName);
                        const ptr = wasmExports.alloc(encoded.length + 1);
                        const mem = new Uint8Array(wasmMemory.buffer);
                        mem.set(encoded, ptr);
                        mem[ptr + encoded.length] = 0;
                        wasmExports.on_display_name_fetched(id, ptr, encoded.length);
                    });
            },
            external_start_fetch_threads: (usernamePtr, usernameLen, offset, size, id) => {
                const username = str(usernamePtr, usernameLen);
                fetch(`/api/user/threads?username=${encodeURIComponent(username)}&offset=${encodeURIComponent(offset)}&size=${encodeURIComponent(size)}`)
                    .then(res => res.json())
                    .then(data => {
                        data.forEach(thread => {
                            let textEncoder = new TextEncoder();
                            function encodeString(str) {
                                const encoded = new TextEncoder().encode(str);
                                const ptr = wasmExports.alloc(encoded.length + 1);
                                const mem = new Uint8Array(wasmMemory.buffer);
                                mem.set(encoded, ptr);
                                mem[ptr + encoded.length] = 0;
                                return (encoded, ptr, encoded.length);
                            }

                            const eUsername, pUsername, lUsername = encodeString(data.Username);
                            const eTitle, pTitle, lTitle = encodeString(data.Title);
                            const eMessage, pMessage, lMessage = encodeString(data.Message);

                            wasmExports.thread_fetched(id, pUsername, lUsername, pTitle, lTitle, pMessage, lMessage, data.Time);
                        });
                        wasmExports.on_threads_fetched(id);
                    })
            }
        }
    }
}
