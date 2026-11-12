from PIL import Image, ImageDraw
from pathlib import Path
root=Path('docs/evidence/native-identity-standardization/roster-family-audit/reviews/garan-rest-family-audit-20261112')
base=Path('assets/chars/sprites/garan')
groups={
 'guard-hit':['04_special','guard','guardHit','crouchGuard','crouchGuardHit','airGuard','airGuardHit','09_hit1','10_hit2','11_hit3','hitL1','hitL2','hitL3','hitCrouch','12_fall1','13_fall2','14_lie'],
 'crouch-cjab-turn':['04_special','01_idle','crouch','crouch0','crouch1','cjab0','cjab1','turn','jumpStart','walk0'],
 'aerial-family':['04_special','jumpStart','jumpUp','jumpTop','jumpDown','hop','flip0','flip1','flip2','flip3','ajab','akick','da0','da1','da2','inline_c75becbcbc592dc9','inline_9d9aa9e1b98a58b0','airLowKick','land']}
cw,ch,cols=402,430,5
for name,stems in groups.items():
 items=[(s,base/(s+'.png')) for s in stems]
 for factor,suffix in [(1,'native'),(2,'nearest-2x')]:
  rows=(len(items)+cols-1)//cols
  sheet=Image.new('RGB',(cw*cols*factor,ch*rows*factor),(45,45,52)); draw=ImageDraw.Draw(sheet)
  for i,(label,path) in enumerate(items):
   im=Image.open(path).convert('RGBA')
   cell=Image.new('RGBA',(400,400),(0,0,0,0)); cell.alpha_composite(im,((400-im.width)//2,400-im.height))
   if factor==2: cell=cell.resize((800,800),Image.Resampling.NEAREST)
   bg=Image.new('RGB',cell.size,(56,56,64)); bd=ImageDraw.Draw(bg); st=16*factor
   for y in range(0,bg.height,st):
    for x in range(0,bg.width,st):
     if (x//st+y//st)%2: bd.rectangle((x,y,x+st-1,y+st-1),fill=(70,70,78))
   bg.paste(cell,(0,0),cell.getchannel('A'))
   x=(i%cols)*cw*factor; y=(i//cols)*ch*factor
   sheet.paste(bg,(x,y+24*factor)); draw.text((x+8*factor,y+5*factor),label,fill='white')
  sheet.save(root/f'{name}-{suffix}.png',optimize=True)
