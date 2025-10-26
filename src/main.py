import time
from machine import Pin
from rotary_irq_esp import RotaryIRQ
import _thread

# Rotary encoder setup (0.5° resolution)
r = RotaryIRQ(pin_num_clk=21,
              pin_num_dt=22,
              min_val=0,
              max_val=720,  # 360° × 2
              reverse=True,
              range_mode=RotaryIRQ.RANGE_BOUNDED,
              half_step=True,
              pull_up=True)

# Output pin
output = Pin(32, Pin.OUT)

# Globals
target_angles = []  # stored in half-steps
current_target_index = 0
encoder_active = False
reset_detected = False
triggered = False
loop_running = False

def rotary_loop():
    global reset_detected, current_target_index, encoder_active, triggered, loop_running
    last_steps = -1
    loop_running = True

    while loop_running:
        if not encoder_active or not target_angles:
            time.sleep(0.2)
            continue

        steps = r._value  # half-step count
        angle = steps / 2.0
        target = target_angles[current_target_index] / 2.0

        # Trigger output only when moving forward from 0° to target
        if not triggered and steps >= target:
            output.on()
            triggered = True
            print(f"⚡ Target reached: {target:.1f}°")

        else:
            output.off()

        # Reset encoder if angle drops below 2°
        if angle < 2 and not reset_detected:
            r.set(0)
            reset_detected = True
            triggered = False
            print("🔄 Encoder reset to 0°")

            # Advance to next target
            current_target_index += 1
            if current_target_index >= len(target_angles):
                print("✅ All targets completed and returned to 0°.")
                stop_encoder()
                loop_running = False
                prompt_for_angles()
                break

        if angle > 5:
            reset_detected = False

        time.sleep(0.05)

def set_angles(cmd):
    global target_angles, current_target_index, encoder_active, loop_running
    try:
        raw = cmd.strip().split(" ", 1)[1]
        angles = [int(float(a.strip()) * 2) for a in raw.split(',') if a.strip()]
        target_angles = angles
        current_target_index = 0
        encoder_active = True
        loop_running = True
        print("🎯 Target angles set:", [a / 2.0 for a in target_angles])
        _thread.start_new_thread(rotary_loop, ())
    except:
        print("❌ Invalid format. Use: set 45,90.5,135")

def stop_encoder():
    global encoder_active
    encoder_active = False
    output.off()
    print("🛑 Encoder stopped.")

def prompt_for_angles():
    print("\n🔁 Enter new angles to restart:")
    while True:
        try:
            cmd = input(">>> ").strip().lower()
            if cmd.startswith("set "):
                set_angles(cmd)
                break
            elif cmd == "stop":
                stop_encoder()
                break
            else:
                print("❓ Unknown command. Use 'set' or 'stop'.")
        except Exception as e:
            print("⚠️ Error:", e)

# Initial prompt
print("\n🔧 ESP32 Rotary Encoder Control")
print("Type a command:")
print("  set 45,90.5,135   → Set target angles (supports decimals)")
print("  stop              → Stop encoder\n")

prompt_for_angles()
