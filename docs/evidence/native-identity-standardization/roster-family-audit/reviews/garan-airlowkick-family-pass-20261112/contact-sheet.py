from PIL import Image, ImageDraw
from pathlib import Path
root=Path('docs/evidence/native-identity-standardization/roster-family-audit/reviews/garan-airlowkick-family-pass-20261112')
base=Path('assets/chars/sprites/garan')
items=[('04_special canonical',base/'04_special.png'),('jumpTop',base/'jumpTop.png'),('akick',base/'akick.png'),('airLowKick',base/'airLowKick.png'),('jumpDown',base/'jumpDown.png')]
cw,ch=502,430
for factor,suf in [(1,'native'),(2,'nearest-2x')]:
  sheet=Image.new('RGB',(cw*3*factor,ch*2*factor),(45,45,52)); draw=ImageDraw.Draw(sheet)
  for i,(label,path) in enumerate(items):
    im=Image.open(path).convert('RGBA')
    if im.width>cw or im.height>400: im.thumbnail((cw,400),Image.Resampling.NEAREST)
    cell=Image.new('RGBA',(cw,400),(0,0,0,0)); cell.alpha_composite(im,((cw-im.width)//2,400-im.height))
    if factor==2: cell=cell.resize((cw*2,800),Image.Resampling.NEAREST)
    bg=Image.new('RGB',cell.size,(56,56,64)); bd=ImageDraw.Draw(bg); step=16*factor
    for y in range(0,bg.height,step):
      for x in range(0,bg.width,step):
        if (x//step+y//step)%2: bd.rectangle((x,y,x+step-1,y+step-1),fill=(70,70,78))
    bg.paste(cell,(0,0),cell.getchannel('A'))
    x=(i%3)*cw*factor; y=(i//3)*ch*factor
    sheet.paste(bg,(x,y+24*factor)); draw.text((x+8*factor,y+5*factor),label,fill='white')
  sheet.save(root/f'contact-{suf}.png',optimize=True)
