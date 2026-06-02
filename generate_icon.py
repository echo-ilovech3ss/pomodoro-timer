import math
from PIL import Image, ImageDraw, ImageFilter

def create_logo_image(size):
    # Create image with transparent background
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    scale = size / 512.0
    
    r_rect = 108 * scale
    border_width = int(3 * scale)
    
    # 1. Draw squircle background (rounded rect)
    sq_rect = [24*scale, 24*scale, 488*scale, 488*scale]
    draw.rounded_rectangle(sq_rect, radius=r_rect, fill=(21, 16, 42, 255), outline=(45, 34, 77, 255), width=border_width)
    
    # Add a subtle radial gradient highlight to the squircle
    glow_img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    glow_draw = ImageDraw.Draw(glow_img)
    glow_draw.circle((size/2, size/2), 160*scale, fill=(180, 100, 255, 25))
    glow_img = glow_img.filter(ImageFilter.GaussianBlur(30*scale))
    img.alpha_composite(glow_img)
    
    # Re-draw border to be crisp on top of glow
    draw.rounded_rectangle(sq_rect, radius=r_rect, fill=None, outline=(45, 34, 77, 255), width=border_width)
    
    # 2. Draw outer ring backdrop
    draw.circle((size/2, size/2), 160*scale, fill=None, outline=(37, 27, 62, 255), width=int(12*scale))
    
    # 3. Draw glowing Pomodoro ring segments (Coral -> Pink -> Purple)
    accent_coral = (255, 110, 80, 255)
    accent_pink = (255, 50, 150, 255)
    accent_purple = (180, 100, 255, 255)
    
    ring_box = [96*scale, 96*scale, 416*scale, 416*scale]
    draw.arc(ring_box, start=180, end=270, fill=accent_coral, width=int(12*scale))
    draw.arc(ring_box, start=270, end=330, fill=accent_pink, width=int(12*scale))
    draw.arc(ring_box, start=330, end=60, fill=accent_purple, width=int(12*scale))
    
    # 4. Draw sound waves (Cozy Lo-Fi Waveforms)
    # Wave 2 (Centered active wave)
    wave_pts = []
    for x in range(int(140*scale), int(372*scale)):
        t = (x - 140*scale) / (232*scale) * 2.0 * math.pi
        y = size/2 + math.sin(t * 1.5) * 45*scale * math.sin(t)
        wave_pts.append((x, y))
    
    if len(wave_pts) > 1:
        draw.line(wave_pts, fill=accent_coral, width=int(8*scale), joint="round")
        
    # Wave 1 (Faded purple background wave)
    wave_pts1 = []
    for x in range(int(160*scale), int(352*scale)):
        t = (x - 160*scale) / (192*scale) * 2.0 * math.pi
        y = size/2 + math.sin(t * 2.0) * 30*scale
        wave_pts1.append((x, y))
    if len(wave_pts1) > 1:
        draw.line(wave_pts1, fill=(180, 100, 255, 120), width=int(5*scale), joint="round")
        
    # Wave 3 (Pink accent wave)
    wave_pts3 = []
    for x in range(int(180*scale), int(332*scale)):
        t = (x - 180*scale) / (152*scale) * 2.0 * math.pi
        y = size/2 - math.sin(t * 1.0) * 20*scale
        wave_pts3.append((x, y))
    if len(wave_pts3) > 1:
        draw.line(wave_pts3, fill=(255, 50, 150, 160), width=int(4*scale), joint="round")

    # 5. Draw central teardrop tomato core
    td_pts = []
    cx, cy = size/2, 235*scale
    rx, ry = 34*scale, 35*scale
    for i in range(100):
        theta = i / 100.0 * 2.0 * math.pi
        r_scale = 1.0 - 0.4 * math.sin(theta/2)
        x = cx + rx * math.cos(theta) * r_scale
        y = cy + ry * math.sin(theta) * r_scale
        if theta > math.pi:
            y += (theta - math.pi) * (2.0 * math.pi - theta) * 1.5 * scale
        td_pts.append((x, y))
        
    draw.polygon(td_pts, fill=(255, 80, 80, 240), outline=(255, 110, 80, 255), width=int(2*scale))
    
    # 6. Draw double leaf accent on top
    # Left leaf
    left_leaf_pts = []
    lcx, lcy = cx, 195*scale
    for i in range(50):
        theta = i / 50.0 * math.pi
        x = lcx - math.sin(theta) * 15*scale
        y = lcy - (1.0 - math.cos(theta)) * 12*scale
        left_leaf_pts.append((x, y))
    for i in range(50):
        theta = i / 50.0 * math.pi
        x = lcx - (1.0 - math.cos(theta)) * 10*scale
        y = lcy - math.sin(theta) * 15*scale
        left_leaf_pts.append((x, y))
    draw.polygon(left_leaf_pts, fill=(80, 220, 140, 255))
    
    # Right leaf
    right_leaf_pts = []
    for i in range(50):
        theta = i / 50.0 * math.pi
        x = lcx + math.sin(theta) * 15*scale
        y = lcy - (1.0 - math.cos(theta)) * 12*scale
        right_leaf_pts.append((x, y))
    for i in range(50):
        theta = i / 50.0 * math.pi
        x = lcx + (1.0 - math.cos(theta)) * 10*scale
        y = lcy - math.sin(theta) * 15*scale
        right_leaf_pts.append((x, y))
    draw.polygon(right_leaf_pts, fill=(60, 180, 110, 255))
    
    return img

sizes = [256, 128, 64, 48, 32, 16]
images = [create_logo_image(s) for s in sizes]

images[0].save("logo.ico", format="ICO", sizes=[(s, s) for s in sizes])
print("Successfully generated logo.ico!")
