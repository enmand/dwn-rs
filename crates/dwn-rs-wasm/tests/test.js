import { TestSuite } from "@tbd54566975/dwn-sdk-js/tests";
import {
  SurrealDataStore,
  SurrealMessageStore,
  SurrealEventLog,
} from "../pkg/index.js";
import WebSocket from "isomorphic-ws";

global.WebSocket = WebSocket;

let s = new SurrealMessageStore();
// await s.connect("ws://192.168.10.56:8000/");
await s.connect("mem://");
let d = new SurrealDataStore();
await d.connect("mem://");
let e = new SurrealEventLog();
await e.connect("mem://");
describe("Store dependent tests", () => {
  TestSuite.runStoreDependentTests({
    messageStore: s,
    dataStore: d,
    eventLog: e,
  });
});
