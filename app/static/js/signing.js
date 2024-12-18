for (let key of Object.keys(DilithiumAlgorithm)) {
    window[key] = DilithiumAlgorithm[key];
}

const level = DilithiumLevel.get(5);
let signaturebis;
function generateDilithiumKeyPair() {

    const keyPair = DilithiumKeyPair.generate(level);

    dilithiumPrivateKey = keyPair.getPrivateKey();
    dilithiumPublicKey = keyPair.getPublicKey();
}

function signDilithium(message) {

    let signature;
    signature = dilithiumPrivateKey.sign(message);
    return signature.toHex();
}


function validateDilithiumSignature(message, signature, user_id) {
    let valid;
    let sourcePublicKey = DilithiumPublicKey.fromHex(dilithium_keys[user_id], level);
    try {
        signature = DilithiumSignature.fromHex(signature, level)
    } catch (ex) {
        alert("Invalid signature provided: " + ex.message);
        console.error(ex);
        return;
    }

    try {
        valid = sourcePublicKey.verifySignature(message, signature);
        return valid;
    } catch (ex) {
        alert("Error: " + ex.message);
        console.error(ex);
        return;
    }
}