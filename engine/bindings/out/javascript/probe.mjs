import { readFile } from 'node:fs/promises';
import { Louder } from './louder/index.js';

const wasm = await readFile(process.argv[2]);
const engine = await Louder.load(wasm);
const r = engine.raw;

const CONFIG = `[{"type":"speaker","name":"A","flags":1}]`;
const DOC = `[
  {"type":"hub","conversation_id":0,"speaker":"A","text":"Hello","tags":"start main","flags":0},
  {"type":"hub","conversation_id":5,"speaker":"A","text":"World","flags":0},
  {"type":"connection","from_id":0,"from_port":0,"to_id":5}
]`;

console.log('config ->', engine.loadConfig(CONFIG));
console.log('conversation ->', engine.loadConversation('c', DOC));
const c = engine.name('c');
const actor = engine.addActor(c, 0, { speakAs: ['A'], listenTo: ['A'] });
console.log('actor ->', actor);

for (let i = 0; i < 4; i += 1) {
  const bytes = r.get_event(engine.ctx, i);
  const v = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  console.log(`get_event(${i}) tag=${v.getInt32(0, true)} size=${bytes.length}`);
}
console.log('events() ->', [...engine.events()]);
engine.close();
