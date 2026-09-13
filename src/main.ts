import { createApp } from "vue";
import App from "./App.vue";
import router from "./core/router.ts";
import * as naive_ui from "naive-ui";

createApp(App)
    .use(router)
    .use(naive_ui)
    .mount("#app");