import { TestSuite } from "@tbd54566975/dwn-sdk-js/tests";
import init, {
  SurrealDataStore,
  SurrealMessageStore,
  SurrealEventLog,
  SurrealResumableTaskStore,
  init_tracing, set_tracing_level, TracingLevel
} from "../browsers/index.js";
import stores from "../browsers/index_bg.wasm";

let instance = await stores();
await init(instance);

set_tracing_level(TracingLevel.Trace);
init_tracing();

let s = new SurrealMessageStore();
// await s.connect("ws://192.168.10.56:8000/");
await s.connect("mem://");
let d = new SurrealDataStore();
await d.connect("mem://");
let e = new SurrealEventLog();
await e.connect("mem://");
let t = new SurrealResumableTaskStore();
await t.connect("mem://");
describe("Store dependent tests", () => {
  TestSuite.runInjectableDependentTests({
    messageStore: s,
    dataStore: d,
    eventLog: e,
    resumableTaskStore: t,
  });
});
