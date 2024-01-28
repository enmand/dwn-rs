import { TestSuite } from "@tbd54566975/dwn-sdk-js/tests";
import { SurrealDB } from "../pkg/index.js";
import WebSocket from "isomorphic-ws";

global.WebSocket = WebSocket;

let s = new SurrealDB();
// await s.connect("ws://192.168.10.56:8000/");
await s.connect("mem://");
describe("Store dependent tests", () => {
  TestSuite.runStoreDependentTests({
    messageStore: s,
  });
});
