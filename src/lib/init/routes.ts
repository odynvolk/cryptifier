import epr from "express-promise-router";
import feed from "../feed";

const router = epr();

router.get("/", feed);

export default router;
