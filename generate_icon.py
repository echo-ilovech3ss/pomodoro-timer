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

    # 5. Finished drawing clean waveforms in center
    pass
    
    return img

sizes = [256, 128, 64, 48, 32, 16]
images = [create_logo_image(s) for s in sizes]

images[0].save("logo.ico", format="ICO", sizes=[(s, s) for s in sizes])
print("Successfully generated abstract logo.ico!")
