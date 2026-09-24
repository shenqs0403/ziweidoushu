import {createRouter, createWebHashHistory} from "vue-router";
import RecorderView from "../views/RecorderView.vue";
import ZiWeiDouShuView from "../views/ZiWeiDouShuView.vue";

const router = createRouter({
    history: createWebHashHistory(),
    routes: [{
        path: "/",
        redirect: "/recorder",
    }, {
        path: "/recorder",
        component: RecorderView,
    }, {
        path: "/ziwei/:id",
        component: ZiWeiDouShuView,
    }]
});
export default router;