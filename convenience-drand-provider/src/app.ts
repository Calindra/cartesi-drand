import { DrandProvider } from "./DrandProvider";
import * as dotenv from "dotenv";

dotenv.config({ debug: true });
new DrandProvider().run()
