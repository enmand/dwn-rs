import { TestSuite } from '@tbd54566975/dwn-sdk-js/tests';
import { SurrealDB } from "../out/index.js";

let s = new SurrealDB();
await s.connect("memory")
describe('Store dependent tests', () => {
    TestSuite.runStoreDependentTests({
        messageStore: s,
    })
})

