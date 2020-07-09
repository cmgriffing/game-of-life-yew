import "./static/style.scss";

const Filter = require('cellulelife-bad-words');
const filter = new Filter();

window.filter = filter;

window.API_URL_SUBMIT_RESULT = process.env.API_URL_SUBMIT_RESULT;
window.API_URL_GET_HIGH_SCORES = process.env.API_URL_GET_HIGH_SCORES;

import("./pkg").then((module) => {
  module.run_app();
});
