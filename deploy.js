var Web3 = require("web3");
var fs = require('fs');
// Connect to our local node
var web3 = new Web3(new Web3.providers.HttpProvider("http://localhost:8545"));

// Setup default account
web3.eth.defaultAccount = "0x004ec07d2329997267ec62b4166639513386f32e";
// Unlock account

web3.eth.personal.unlockAccount(web3.eth.defaultAccount, "user");

// read JSON ABI
var abi = JSON.parse(fs.readFileSync("./target/json/DonationContract.json"));
// convert Wasm binary to hex format
var codeHex = '0x' + fs.readFileSync("./target/pwasm_tutorial_contract.wasm").toString('hex');

var DonationContract = new web3.eth.Contract(abi, { data: codeHex, from: web3.eth.defaultAccount, gas: 10000000 });

// Will create DonationContract
DonationContract
	.deploy({data: codeHex, arguments: []})
	.send({from: web3.eth.defaultAccount})
	.error((e) => console.log(e))
	.then((a) => console.log(a));