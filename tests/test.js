import { TestSuite } from '@tbd54566975/dwn-sdk-js/tests';
import { SurrealDB } from "../out/index.js";

let s = new SurrealDB();
await s.connect("mem://test.db");
await s.with_tenant("test");

TestSuite.runStoreDependentTests({
    messageStore: s,
})
