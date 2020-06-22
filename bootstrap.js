import "./static/style.scss";

window.API_URL_SUBMIT_RESULT = process.env.API_URL_SUBMIT_RESULT;
window.API_URL_GET_HIGH_SCORES = process.env.API_URL_GET_HIGH_SCORES;

import("./pkg").then((module) => {
  module.run_app();
});
