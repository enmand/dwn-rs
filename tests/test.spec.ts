import { Dwn, DataStoreLevel, EventLogLevel } from '@tbd54566975/dwn-sdk-js';
import { SurrealDB } from "../out/index.js";

// (async () => {
//     let s = new SurrealDB();
//     try {
//         await s.connect("mem://test.db");
//         await s.put("test", { descriptor: { interface: 'interface', method: 'method', messageTimestamp: '0' } }, { "test": true });
//         let m = await s.query("test", { "test": true })
//         console.log(m);
//     } catch (e) {
//         console.log(e);
//     }
//
//     TestSuite.runStoreDependentTests({
//         messageStore: s,
//     });
// })();
//

test("dwn-sdk-js", async () => {
    Dwn.create({
        messageStore: new SurrealDB(),
        dataStore: new DataStoreLevel(),
        eventLog: new EventLogLevel(),
    })
});

console.log("done running");
