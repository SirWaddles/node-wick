const wick = require("./index");
const fs = require("fs");

let uasset_buf = fs.readFileSync("./test_assets/uasset.uasset");
let uexp_buf = fs.readFileSync("./test_assets/uexp.uexp");
let ubulk_buf = fs.readFileSync("./test_assets/ubulk.ubulk");

let asset = new wick.Package(uasset_buf, uexp_buf, ubulk_buf);
fs.writeFileSync("./test_assets/test.png", asset.get_texture());
