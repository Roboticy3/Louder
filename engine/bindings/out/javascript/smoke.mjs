import { readFile } from 'node:fs/promises';
import { Louder } from './louder/index.js';

const CONFIG = `[
  {"type":"speaker","name":"A","flags":1},
  {"type":"speaker","name":"B","flags":0},
  {"type":"speaker","name":"Quiet","flags":0,"listening":"A\\nB"}
]`;

const CONVERSATION = `[
  {"type":"hub","conversation_id":0,"speaker":"A","text":"Hello","tags":"start main","flags":0},
  {"type":"hub","conversation_id":5,"speaker":"A","text":"World","flags":0},
  {"type":"hub","conversation_id":9,"text":"The end","callbacks":"exit","flags":0},
  {"type":"connection","from_id":0,"from_port":0,"to_id":5},
  {"type":"connection","from_id":5,"from_port":0,"to_id":9}
]`;

const failures = [];
function check(condition, label) {
  console.log((condition ? '  ok   ' : '  FAIL ') + label);
  if (!condition) failures.push(label);
}

const wasm = await readFile(process.argv[2]);

console.log('names');
{
  const engine = await Louder.load(wasm);
  check(engine.text(engine.name('alpha')) === 'alpha', 'a name round-trips');
  check(engine.lookup('never declared') === engine.constructor ? true : true, 'placeholder');
  check(engine.lookup('never declared') === -1, 'an undeclared name does not resolve');
  check(engine.text(-1) === '', 'the empty name reads as empty');
  engine.close();
}

console.log('documents');
{
  const engine = await Louder.load(wasm);
  check(engine.loadConfig(CONFIG) === 'Ok', 'the speaker config loads: ' + engine.loadConfig(CONFIG));
  check(engine.loadConversation('under test', CONVERSATION) === 'Ok', 'the conversation loads');
  check(engine.loadConversation('under test', CONVERSATION) !== 'Ok', 'loading twice is refused');
  check(engine.loadConversation('garbage', 'not json at all') === 'MalformedJson', 'malformed JSON is refused');

  const c = engine.name('under test');
  check(engine.hubText(c, 0) === 'Hello', 'hub 0 reads back');
  check(engine.hubText(c, 9) === 'The end', 'hub 9 reads back');
  check(engine.hubSpeaker(c, 0) === 'A', 'hub 0 is spoken by A');
  check(engine.hubCallbacks(c, 9) === 'exit', 'hub 9 carries a callback');

  const hubs = [...engine.hubByChannel(c, 'main')];
  check(JSON.stringify(hubs) === '[0,5,9]', 'hubs iterate in ascending order: ' + JSON.stringify(hubs));

  const actor = engine.addActor(c, 0, { speakAs: ['A'], listenTo: ['A'] });
  check(actor === 0, 'the first actor is 0');
  check(engine.actorPosition(c, actor) === 0, 'the actor starts at 0');
  check(engine.hubChoiceCount(c, 0, actor) === 1, 'hub 0 has one choice');
  check(engine.hubChoiceText(c, 0, actor, 0) === 'World', 'the choice reads as the next hub');

  const kinds = [...engine.drainEvents()].map((e) => e.kind);
  check(kinds.includes('ActorAdded'), 'the actor-added event arrived: ' + JSON.stringify(kinds));

  check(engine.choose(c, actor, 0), 'the choice is taken');
  check(engine.actorPosition(c, actor) === 5, 'the actor moved to hub 5');
  check(engine.warp(c, actor, 0), 'the actor is warped back');
  check(engine.actorPosition(c, actor) === 0, 'the warp landed');
  check(engine.teleport(c, actor, 9), 'the actor is teleported');
  check(engine.actorPosition(c, actor) === 9, 'the teleport landed');

  const speaking = [...engine.actorSpeakingAs(c, 'A')];
  check(JSON.stringify(speaking) === '[0]', 'speaking-as iteration finds the actor: ' + JSON.stringify(speaking));

  engine.set(c, 'mood', 7);
  check(engine.get(c, 'mood') === 7, 'state reads back');
  check(engine.get(c, 'unset') === 0, 'an unwritten variable reads as 0');
  engine.set(c, 'who', 'A');
  check(engine.get(c, 'who') === 'A', 'a text variable round-trips through a name');
  engine.set(c, 'ratio', 0.5);
  check(engine.get(c, 'ratio') === 0.5, 'a float variable round-trips');
  check(engine.stateJson(c).replace(/\s/g, '').includes('"mood":7'), 'the state JSON carries the value: ' + engine.stateJson(c));
  check(engine.snapshotJson(c).includes('actors'), 'the snapshot carries the actors');
  engine.close();
}

console.log('events, decoded');
{
  const engine = await Louder.load(wasm);
  engine.loadConfig(CONFIG);
  engine.loadConversation('c', CONVERSATION);
  const c = engine.name('c');
  const actor = engine.addActor(c, 0, { speakAs: ['A'], listenTo: ['A'] });
  engine.choose(c, actor, 0);
  const seen = {};
  for (const event of engine.drainEvents()) seen[event.kind] = event.value;
  console.log('   ', JSON.stringify(seen));
  const motion = seen.RegularMotion ?? seen.Warp;
  check(motion !== undefined, 'a motion event arrived');
  check(motion && motion.sourcePosition === 0 && motion.targetPosition === 5, 'the motion is decoded: ' + JSON.stringify(motion));
  engine.close();
}

console.log('raw layer');
{
  const engine = await Louder.load(wasm);
  check(engine.call('test_linkage') === 123, 'an unwrapped entry point is reachable');
  engine.close();
}

console.log();
if (failures.length) {
  console.log(failures.length + ' FAILURES:');
  for (const name of failures) console.log('  ' + name);
  process.exit(1);
}
console.log('all checks passed');
