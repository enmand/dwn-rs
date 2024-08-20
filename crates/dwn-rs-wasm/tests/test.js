import { TestSuite } from "@tbd54566975/dwn-sdk-js/tests";
import {
  SurrealDataStore,
  SurrealMessageStore,
  SurrealEventLog,
  SurrealResumableTaskStore,
  EventStream,
  init_tracing, TracingLevel,
} from "../pkg/index.js";
import WebSocket from "isomorphic-ws";

global.WebSocket = WebSocket;

init_tracing(TracingLevel.Error);

let s = new SurrealMessageStore();
await s.connect("mem://");
let d = new SurrealDataStore();
await d.connect("mem://");
let e = new SurrealEventLog();
await e.connect("mem://");
let t = new SurrealResumableTaskStore();
await t.connect("mem://");
let es = new EventStream();
describe("Store dependent tests", () => {
  TestSuite.runInjectableDependentTests({
    messageStore: s,
    dataStore: d,
    eventLog: e,
    eventStream: es,
    resumableTaskStore: t,
  });
});
