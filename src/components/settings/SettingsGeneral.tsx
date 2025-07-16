import { useNavigate } from 'react-router-dom';
import { useState, useEffect } from "react";
import { invoke } from '@tauri-apps/api/core';


type privacy = {
    encryption: string,
    signature: string,
    key_exchange: string,
    chat_deletion_timer: number
}

type settings = {
    name: string,
    user_id: string,
    privacy_settings: privacy,
}

export default function SettingsGeneral() {
    const navigate = useNavigate();
const [proxyType, setProxyType] = useState("None");
const [socksHost, setSocksHost] = useState("");
const [socksPort, setSocksPort] = useState("");
const [useAuth, setUseAuth] = useState(false);
const [username, setUsername] = useState("");
const [password, setPassword] = useState("");
    useEffect(() => {
        (async () => {
          let settings: settings = await invoke("get_profile_settings");
          console.log(settings)
          setSignatureType(settings.privacy_settings.signature);
          setEncryptionType(settings.privacy_settings.encryption);
          setKeyExchangeType(settings.privacy_settings.key_exchange);
        })();
      }, []);

    const [signatureType, setSignatureType] = useState("");
    const [encryptionType, setEncryptionType] = useState("");
    const [keyExchangeType, setKeyExchangeType] = useState("");

      return (
    <div className="flex flex-col w-screen h-screen  bg-neutral-950 text-neutral-100 opacity-0 animate-fade-in">
        <div className="flex items-center font bg-neutral-900 border-b border-neutral-800">
            <svg className="m-4 size-8" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" onClick={() => {navigate('/')}}>
              <path fillRule="evenodd" d="M7.28 7.72a.75.75 0 0 1 0 1.06l-2.47 2.47H21a.75.75 0 0 1 0 1.5H4.81l2.47 2.47a.75.75 0 1 1-1.06 1.06l-3.75-3.75a.75.75 0 0 1 0-1.06l3.75-3.75a.75.75 0 0 1 1.06 0Z" clipRule="evenodd" />
            </svg>
            <div className="m-4 font-bold">General</div>
        </div>
        <div className="flex-1">
            {/* TLS settings */}
            <>
            <div className="mx-6 flex flex-row h-16 items-center font-bold">
                TLS settings
            </div>
            <div className='mx-6 text-xs text-red-600'>
                Disclamer : Oblivion uses a custom TLS implementation to allow for post-quantum-cryptography.
            </div>
            <div className='mx-6 text-xs text-red-600'>
                It contains a few know vunerabilities patched in normal TLS, but it is still secure enough for everyday use.
            </div>
            <div className='mx-6 text-xs text-red-600'>
                The user should do research on the chosen algorithms before using them to make sure they are ok with the trade-offs.
            </div>
            <div className="flex flex-col">
              <div className="flex flex-row h-16 items-center">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="size-8 mx-6 text-neutral-500">
                  <path fillRule="evenodd" d="M18.685 19.097A9.723 9.723 0 0 0 21.75 12c0-5.385-4.365-9.75-9.75-9.75S2.25 6.615 2.25 12a9.723 9.723 0 0 0 3.065 7.097A9.716 9.716 0 0 0 12 21.75a9.716 9.716 0 0 0 6.685-2.653Zm-12.54-1.285A7.486 7.486 0 0 1 12 15a7.486 7.486 0 0 1 5.855 2.812A8.224 8.224 0 0 1 12 20.25a8.224 8.224 0 0 1-5.855-2.438ZM15.75 9a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0Z" clipRule="evenodd" />
                </svg>
                <div className="">Signature</div>
              </div>
              <div className="ml-20 w-48 mb-2">
                <select
                  value={signatureType}
                  onChange={(e) => setSignatureType(e.target.value)}
                  className="appearance-none w-full px-4 py-2 rounded-md bg-neutral-900 text-neutral-100 border border-neutral-700 focus:outline-none focus:ring-2 focus:ring-primary-500"
                >
                  <option value="Dilithium-Ed25519">Dilithium-Ed25519</option>
                  <option value="Falcon-Ed25519">Falcon-Ed25519</option>
                </select>
              </div>
            </div>
            <div className="flex flex-col">
                <div className="flex flex-row h-16 items-center">
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="size-8 mx-6 text-neutral-500">
                      <path d="M21.731 2.269a2.625 2.625 0 0 0-3.712 0l-1.157 1.157 3.712 3.712 1.157-1.157a2.625 2.625 0 0 0 0-3.712ZM19.513 8.199l-3.712-3.712-12.15 12.15a5.25 5.25 0 0 0-1.32 2.214l-.8 2.685a.75.75 0 0 0 .933.933l2.685-.8a5.25 5.25 0 0 0 2.214-1.32L19.513 8.2Z" />
                    </svg>
                    <div className=''>Key exchange</div>
                </div>
                <div className="ml-20 w-48 mb-2">
                <select
                  value={keyExchangeType}
                  onChange={(e) => setKeyExchangeType(e.target.value)}
                  className="appearance-none w-full px-4 py-2 rounded-md bg-neutral-900 text-neutral-100 border border-neutral-700 focus:outline-none focus:ring-2 focus:ring-primary-500"
                >
                  <option value="Kyber-1024">Kyber-1024</option>
                  <option value="Kyber-768">Kyber-768</option>
                  <option value="Kyber-512">Kyber-512</option>
                  <option value="FrodoKEM-1344">FrodoKEM-1344</option>
                  <option value="FrodoKEM-976">FrodoKEM-976</option>
                  <option value="FrodoKEM-640">FrodoKEM-640</option>
                </select>
              </div>
            </div>
            <div className="flex flex-col">
                <div className="flex flex-row h-16 items-center">
                    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" className="size-8 mx-6 text-neutral-500">
                      <path fillRule="evenodd" d="M12 1.5a5.25 5.25 0 0 0-5.25 5.25v3a3 3 0 0 0-3 3v6.75a3 3 0 0 0 3 3h10.5a3 3 0 0 0 3-3v-6.75a3 3 0 0 0-3-3v-3c0-2.9-2.35-5.25-5.25-5.25Zm3.75 8.25v-3a3.75 3.75 0 1 0-7.5 0v3h7.5Z" clipRule="evenodd" />
                    </svg>

                    <div className=''>Symetric encryption</div>
                </div>
                <div className="ml-20 w-48 mb-2">
                <select
                  value={encryptionType}
                  onChange={(e) => setEncryptionType(e.target.value)}
                  className="appearance-none w-full px-4 py-2 rounded-md bg-neutral-900 text-neutral-100 border border-neutral-700 focus:outline-none focus:ring-2 focus:ring-primary-500"
                >
                  <option value="AES-GCM">AES-GCM</option>
                  <option value="ChaCha20">ChaCha20</option>
                </select>
              </div>
            </div>
            </>
            {/* Proxy settings */}
<div className="flex flex-row h-16 items-center font-bold border-t border-neutral-800">
  <div className="mx-6">
    Proxy
  </div>
</div>

<div className="flex flex-col mx-6 gap-2">

  <select
    value={proxyType}
    onChange={(e) => setProxyType(e.target.value)}
    className="w-48 px-4 py-2 rounded-md bg-neutral-900 text-neutral-100 border border-neutral-700 appearance-none"
  >
    <option value="None">None</option>
    <option value="Tor">Tor</option>
    <option value="SOCKS4">SOCKS4</option>
    <option value="SOCKS5">SOCKS5</option>
  </select>

  {(proxyType === "SOCKS4" || proxyType === "SOCKS5") && (
    <>
      <div className="flex gap-2 ">
        <input
          type="text"
          value={socksHost}
          onChange={(e) => setSocksHost(e.target.value)}
          placeholder="Host"
          className="w-32 px-2 py-1 rounded-md bg-neutral-900 text-neutral-100 border border-neutral-700"
        />
        <input
          type="text"
          value={socksPort}
          onChange={(e) => setSocksPort(e.target.value)}
          placeholder="Port"
          className="w-20 px-2 py-1 rounded-md bg-neutral-900 text-neutral-100 border border-neutral-700"
        />
      </div>

      <label className="flex items-center gap-2 mt-2 text-sm">
        <input
          type="checkbox"
          checked={useAuth}
          onChange={(e) => setUseAuth(e.target.checked)}
          className="form-checkbox accent-primary-500"
        />
        Use Authentication
      </label>

      {useAuth && (
        <div className="flex flex-col gap-1">
          <input
            type="text"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            placeholder="Username"
            className="w-48 px-2 py-1 rounded-md bg-neutral-900 text-neutral-100 border border-neutral-700"
          />
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            placeholder="Password"
            className="w-48 px-2 py-1 rounded-md bg-neutral-900 text-neutral-100 border border-neutral-700"
          />
        </div>
      )}
    </>
  )}
</div>

        </div>
    </div>
  );
}