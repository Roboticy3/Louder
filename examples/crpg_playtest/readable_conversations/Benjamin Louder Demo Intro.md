% using this file will require features #23, #89, #90, #91
% but I think its worth the investment since it pretty much removes the need for a dedicated editor.

Heart
Another day, another ring of wood on the old arteries!

Rhythm
You are a young adult. Probably in your early 20s.

tags: 0_center
Future
It happened again. An image of your life collapsing inward in front of you. 
> 0_completely_blind
> 0_panic
> 0_delusion
> 0_health


tags: 0_completely_blind
Benjamin
What's going on? Who are you?

Rhythm
We're aspects of you. Currently, you are trapped in your own head. Good luck.

link: 0_center

tags: 0_panic
Benjamin
Help! I'm 20-something and I'm not doing anything with my life!

Future
Yes. That is why you came here. 

Benjamin
Where is that?

Memory
The Rouish holy site of East Breach. 
> 0_query_memory
> 0_query_action
> 0_query_annoyed

tags: 0_query_memory
Benjamin
Oh cool, my memory! What else can you tell me?

tags: 1_query_memory
Memory
The sky is blue, the desert is dry. The stars are bright. I know their names. As well as the names of some earthly places. I'm even proficient at multiplication and division, if you ask nicely.
> 1_argue_memory
> 1_contradiction
> 1_u_stoopid
> 1_continue

tags: 1_argue_memory
Benjamin
How is that supposed to help me? What about my name? What I was doing yesterday?

link: 1_query_memory
Memory
Sorry. But at your own request I've blocked out: personally identifyable information (PII), sensitive information (SI), and non-public customer information (NCI). I don't even know if it's still there.

tags: 1_contradiction
Benjamin
Earlier, you stated my age. If you know that, why not anything else?

Memory
That was not a memory. I do not know how it happened. I would imagine you simply observed how old you were from the number of aches in your back. Check the conversation log.

expressions: push_tutorial
Tutorial
Click the journal icon in the bottom right to view a log of what you've said and heard.
> 1_contradiction_persist
> 1_contradiction_yield

tags: 1_contradiction_persist
Benjamin
I highly doubt I could know my age just from *being.*

link: 1_query_memory
Memory
Bad guess. Sorry.

tags: 1_contradiction_yield
flags: 1
Benjamin
Fine.

link: 1_query_memory

tags: 1_u_stoopid
Benjamin
Oh yeah? What's 9 + 10

Memory
9 + 10 is 19. And 384 times 457 is 163149.
> 1_query_memory

tags: 1_continue
expressions: pop_tutorial
Benjamin
Don't talk to me until you get your act together.
> 0_center

tags: 1_continue
expressions: pop_tutorial
Benjamin
Ok. I'm done here.
> 0_center

tags: 0_delusion
Benjamin
You sound delusional.

tags: 2_delusion
Future
Oh, I'm lusional, big guy. Check the name. Whether you like it or not, I'll tell you what's about to happen. You can prepare, or die. But preventing The Future is typically does not work. 
> 2_prove_it
> 2_explain
> 2_doubt
> 2_continue

tags: prove_it
Benjamin
Prove it.

expressions: push_tutorial
Tutorial
Some dialogue options involve rolling 2 six sided dice, plus a skill bonus, against a "check" or level. If the roll meets or surpasses the level, one outcome will occur. If the roll does not meet the level, another outcome will occur. Some checks appear white. These can be retried upon increasing the appropriate skill and by other means. Others are black, and cannot be retried.

flags: 1
> 2_prove_it_success
> 2_prove_it_fail

tags: 2_prove_it_success
expressions: check("Future", 10)
Future
Soon, you will open your eyes.
> eyes_open
> 2_reject_future

tags: 2_reject_future
Benjamin
Obviously, yeah. I don't think that was an effective demonstration. 

Future
But, technically, it was correct, no?

Tutorial
Dice checks will cause a green flash on success, and a red flash on failure.

Benjamin
My brain is useless...

Future
Believe me, I come in handy when I need to.
> 2_success_tutorial

tags: 2_reject_future
Benjamin
No. I'm going to keep my eyes shut forever and never open them! Take that!

tags: 2_success_tutorial
Tutorial
Dice checks will cause a green flash on success, and a red flash on failure. You will, in fact, open your eyes soon.
> 2_delusion

link: 2_delusion
Future
I wish you the poorest of luck.

tags: 2_prove_it_fail
Future
You will liberate your people, soon.
> 2_fail_sure
> 2_fail_reject

tags: 2_sure
Benjamin
Good

Future
Remain dedicated to your true cause, and it *will* happen.
> 2_fail_lasso

tags: 2_fail_reject
Benjamin
What the hell are you talking about?

Future
You don't remember? Your true purpose?

Benjamin
No.

Future
Oh.
>2_fail_lasso

tags: 2_fail_lasso
Tutorial
Dice checks will cause a red flash when they fail. This check failed. But failure may be just as interesting or fruitful as success.

link: 2_delusion
Future
What else would you like to know about The Future?

tags: 2_continue
expressions: pop_tutorial
Benjamin
That's enough future sight for today.

link: 0_center

tags: 0_query_action
Benjamin
What am I doing here?

tags: 3_query_action
Planets
Rediscovering yourself.

Stars
Saving the world.

Heart
Oh great. These jokers. Don't listen to them. You have a headache. Your hands are bound. Judging from the aura of chemicals around you..

Rhythm
It was some party!

Heart
You're hungover. And probably under arrest.

tags: 0_health
Benjamin
Wait so why *does* my chest hurt then?

Heart
Stress? I don't know. The clock is never wrong. You are young, but I feel like I'm aging before my time. My health recommendations are to take it easy from now on.

Tutorial
Currently, you are feeble. You can receive damage both in terms of health and morale. If you run out of either, you will be out of commission until you are found by another player or, if you are hosting, you reload your save.

link: 0_center

tags: pre_open_eyes
expressions: all_tutorials
Planets
Everybody look out! Here comes a circus box of a man: 6 men in 1! Encased in wooden skin and cloudy glass eyes. How does he do it? How does he keep it together? Nobody knows!

tags: open_eyes
expressions: open_eyes
Stars
The sky floods into your vision first. At the border of the heavens, starlight condenses into sky and floats down, eventually precipitating into sand, where it is touched by the sun, and water, where it is touched by the moon. From which are born all other things.

Freya
Your hands are bound, and you are under the watch of an office of the Guard of East Breach. Two things have stuck from before you escaped into your mind: She had introduced herself as Freya and won't let you out of her sight until you identify yourself. See the problem?

expression: end
