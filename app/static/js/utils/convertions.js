export function stringToUint8Array(plaintext) {
    const encoder = new TextEncoder();
    return encoder.encode(plaintext);
}

export function uint8ArrayToString(array) {
    const decoder = new TextDecoder();
    return decoder.decode(array);
}

export function convertToHex(key) {
    let hex = '';
    for (let i = 0; i < key.length; i++) {
        hex += key[i].toString(16).padStart(2, '0');
    }
    return hex;
}


export function hexToUint8Array(hex) {
    const bytes = new Uint8Array(hex.length / 2);
    for (let i = 0; i < hex.length; i += 2) {
        bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
    }
    return bytes;
}

export function hexToArrayBuffer(hex) {
    if (typeof hex !== 'string') {
        throw new TypeError('Expected a string for hex input.');
    }

    const array = new Uint8Array(hex.length / 2);
    for (let i = 0; i < hex.length; i += 2) {
        array[i / 2] = parseInt(hex.substring(i, i + 2), 16);
    }
    return array.buffer;
}

export function arrayBufferToHex(buffer) {
    return Array.from(new Uint8Array(buffer))
        .map(byte => byte.toString(16).padStart(2, '0'))
        .join('');
}