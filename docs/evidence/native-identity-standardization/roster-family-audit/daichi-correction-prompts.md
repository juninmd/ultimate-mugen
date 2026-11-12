# Daichi confirmed sprite correction prompts

Generate exactly one standalone combat sprite per call. Attach assets/chars/sprites/daichi/04_special.png first as the sole authority for Daichi identity, proportions, palette and pixel-art finish. Attach the named target second for pose guidance only. Preserve that pose, direction and silhouette while using the canonical white mane, sleeveless green gi, cream trousers, dark boots and matching white wraps; replace the long-sleeved green gi and gray-trouser outfit. Do not add effects, glow, scenery, text or shadows unless present in the target pose. Use hard-edged pixel clusters, no antialiasing, transparent background, full body, and normalize to 400×400. Preserve the target's meta.json geometry and compare the candidate with its identity anchor and neighboring poses at native and enlarged sizes. Map each generated result only to its named target; keep source and normalized candidate separate until reviewed.

## One-pose targets

| Target file | Affected animations | Current SHA-256 |
| --- | --- | --- |
| `cjab0.png` | cjab | `79a6a313ac699f4b581733f8d2d3fd6facbbdd3ccdcd1e104af388c07fd2c62f` |
| `cjab1.png` | cjab | `ca38ad002873945e4d3b80df7ec2920528646add646bff31827f692585938145` |
| `clk.png` | clk | `29b3143f5c43ba5a55caf6ede2b93d9843cb34b5ba83e3fe4ea836feae203106` |
| `crouch0.png` | crouchEnd, crouchStart, lowKick | `955ee7dc42e855a23ac1de43f2f8b01b0ab33042219b3c38cd38fe4e096b1c0c` |
| `crouch1.png` | clk, crouch, lowKick | `9ef1ce55812e74cc568495389cafd4d66c11b4ac0a54afd6aed1aa2925e485bb` |
| `dfKneel0.png` | defeat3 | `7df3496b32dcc970026f3f8b02a6e3675d7915f3ce4138d2368a7f1c4f44e616` |
| `dfKneel1.png` | defeat3 | `277b6840c28589f136cc21b8125d8acbe44cf75635e9686e361421868f86005c` |
| `dfProp.png` | defeat1 | `1e2680b0b107e4c697e1fd37b79f56f962782888fc3db9e4844483e66d465425` |
| `dfProp2.png` | defeat1 | `5933cd9d385d08550530fe7125102fbb6735b16d4ba1dcb23273d0d04ce51292` |
| `dfSit0.png` | defeat2 | `64952fea09ab548080e7a585c9e9e87775b871e387f7d21e6875a151126aaa16` |
| `dfSit1.png` | defeat2 | `1242174a40603a96fc3548d91d7e612e457c98a9036154bef9164ef105e20525` |
| `flip1.png` | airRecover, jumpFwd | `14777edbf8c8286ccf53273fa21c0ffef60903de43cce3a4d45636eb97bdb48e` |
| `flip2.png` | airRecover, jumpFwd | `d012ad7a366b4e61348c6518ca880de9d0368515cbb103982efd9e44343d4514` |
| `flip3.png` | airRecover, jumpFwd | `778e88709d4e03f88632efb4847c721dc3b2eb881dbde13daf8e0956254874fe` |
| `getUp2.png` | getUp | `f2f7a45c13a700f95568b1f4765d444c5bc6793ac2404ee81f95cea9fb8bfe78` |
| `getUp3.png` | getUp | `955ee7dc42e855a23ac1de43f2f8b01b0ab33042219b3c38cd38fe4e096b1c0c` |
| `hitCrouch.png` | hitCrouch | `069c6d2ab60df6a32878e58b669b386b7fb5b39e79ef930095f9b6f397180489` |
| `hop.png` | hop | `462b0262ff97dde09163f17269e035335b87a9f0067f1869d65aa5a386cdb737` |
| `jumpDown.png` | airLowKick, airRecover, ajab, akick, daybreak, daybreakEx, jumpDown, summitDrop | `688c2f727a75b8f1bfc7b561ef45a7822f3ac2417a4749fe2d00442688ed429f` |

Begin with one representative target as a pilot. Review it before generating the remaining poses.
