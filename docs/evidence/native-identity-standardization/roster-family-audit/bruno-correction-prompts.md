# Bruno confirmed sprite correction prompts

Generate exactly one standalone combat sprite per call. Attach assets/chars/sprites/bruno/04_special.png first as the sole authority for Bruno identity, proportions, palette and pixel-art finish. Attach the named target second for pose guidance only. Preserve that pose, direction and silhouette while using the canonical short brown hair, bare torso, red boxing trunks with gold trim, gold championship belt, white hand wraps and black lace-up boxing boots; replace the gray-top/red-pants and alternate dark-haired boxer designs. Do not add effects, glow, scenery, text or shadows unless present in the target pose. Use hard-edged pixel clusters, no antialiasing, transparent background, full body, and normalize to 400×400. Preserve the target's meta.json geometry and compare the candidate with its identity anchor and neighboring poses at native and enlarged sizes. Map each generated result only to its named target; keep source and normalized candidate separate until reviewed.

## One-pose targets

| Target file | Affected animations | Current SHA-256 |
| --- | --- | --- |
| `br0.png` | bullrush, slam, titanSuplex, titanSuplexUlt | `61a1cacf970d6d2e67eeb0f39e4d52b4087a478215d5170712051190edba63d9` |
| `br1.png` | bullrush, titanSuplex, titanSuplexUlt | `8f054a299aeb0a3a580579a95e22ad71cf3b0a465817725d989c3b959a1c6255` |
| `crouchGuard.png` | crouchGuard, crouchGuardHit, crouchGuardStart | `e9e01e9f73713c9dcd3a1e4e71dcfddc99fdeae0a77de1cbe1b137b719e7fbb5` |
| `crouchGuardHit.png` | crouchGuardHit | `ad68406909757df571c74af7079a08b8d4d01c76fbb7b316be8bb925000321e6` |
| `dfKneel0.png` | defeat3 | `f96364bfb280b50184ae3a7ab423bd83368812af5fa6423d22d4758324ffa986` |
| `dfKneel1.png` | defeat3 | `a0fc0528db25a49adee85af2a40378a5a653e98cb2dd58a7d3a78f72d57f3b7c` |
| `dfSit0.png` | defeat2 | `35e0ef0728cd55c6ec7422b43348355483d727c3066c8a5e6cb63d9ed9d9afce` |
| `dfSit1.png` | defeat2 | `f186984617fa5227652450f2af4f45ce48ef744b414ec78bedab62695a2bba92` |
| `flip0.png` | airRecover, jumpFwd | `4a0e973e502d5c390ff221f0a3e8ffdf31dc75b875e2d4788753dbb4008399a6` |
| `flip1.png` | airRecover, jumpFwd | `fa8f0953055a213df656d73d01697e41f2896368f03b87ed17c50eaa03b59ea1` |
| `flip2.png` | airRecover, jumpFwd | `7ac29007be0d9fdcb7d1a295ad338b5c831c1ddb2a0d4a4bb171bf015c8408ae` |
| `flip3.png` | airRecover, jumpFwd | `17490f1399cc9ff21e9e8045261ac45e12a299a9a9b02e64d9e44d5f0d1ee211` |
| `getUp1.png` | getUp | `ec831284741517ea4cca8d1422ce138c2e2a54efedd88da91c303fc505fc5acf` |
| `getUp2.png` | getUp | `e9aa88f17d2dd01f7912defe37cd55218492a442bd3f3e7aad63520b8fa295c3` |
| `hitCrouch.png` | hitCrouch | `d6ea061bf10a9bbe6c42c00dd96122867fb797e33d73d0f10c8de47b3df6c8ec` |
| `hk0.png` | hkick, stringC | `68cd72c727a3d00a68beafba5693643e03918b91f17898ba8cf79bde3accaeab` |
| `hk1.png` | hkick, stringC | `39048d5e24a17492bd28bfe14150f292f94d23178f718dd2f1d4dd2bcd3c88cf` |
| `hop.png` | hop | `75e3bdbc91e4259dd816e5ca564a4e9d01e00c69ab563a421e41e86edb36bfe3` |
| `jumpDown.png` | airLowKick, airRecover, ajab, akick, cornerDive, jumpDown, lariat, lariatEx | `cd01fb1805a4f7b7f08fd33051b7ebc2df9ff8b3ea0bd13792ce586bd3e85ae1` |
| `land.png` | land | `b79085e802a4f1f9b5ec2dd70d635e9802f5a04fe0d6f58cc0b951d123328d74` |
| `qs0.png` | quake, quakeEx | `b45e5f2b9ca4f943e9352eba1e051163cabb7cfc05447641c35fa69cc3624c08` |
| `qs1.png` | quake, quakeEx | `8f24aa4f2a840138431b160836f8e75de8a0976284b1585627cfa7462f922ece` |
| `sl0.png` | ironClaw, lariat, lariatEx, slam, titanSuplex, titanSuplexUlt | `2fa07b80631a2ff411df4c1e50a5bb4dfd5f2e430e45a3c4ae685615126b1748` |
| `sl1.png` | ironClaw, lariat, lariatEx, slam, titanSuplex, titanSuplexUlt | `aa681b39e43ab4bfb242b479f8f16b0b04a342dcf934e81a1d24b58562114042` |
| `sl2.png` | ironClaw, lariat, lariatEx, slam, titanSuplex, titanSuplexUlt | `0e958a6173b8cf3a1d5943922a345427f3e158482a8305d3aec7457d0848cf12` |
| `sweep0.png` | sweep | `cc24ade3882fa6fb3b96385f389113857677cdafcd4622672af4a4e01c5e8601` |
| `sweep1.png` | sweep | `325a10dbbe1d0dc085a85e728aa4367e71c7660e37555ab1b2759fd5068efd12` |
| `turn.png` | turn | `5b0e995dd081816674b3e49503cb0138fbb60fef18b655bf8d047a97fb66d2e9` |

Begin with one representative target as a pilot. Review it before generating the remaining poses.
