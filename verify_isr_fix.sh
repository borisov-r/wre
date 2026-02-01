#!/bin/bash
# Verification script to check if ISR fix is present in code

echo "=================================="
echo "ISR Fix Verification Script"
echo "=================================="
echo ""

echo "Checking for critical code patterns..."
echo ""

# Check 1: Subscription handle variables
echo "1. Checking for subscription handle variables:"
if grep -q "let _clk_subscription;" src/main.rs && grep -q "let _dt_subscription;" src/main.rs; then
    echo "   ✓ PASS: Subscription handle variables declared"
    grep -n "let _.*_subscription;" src/main.rs | head -2
else
    echo "   ✗ FAIL: Subscription handle variables NOT found!"
    echo "   → This is likely the problem!"
fi
echo ""

# Check 2: Handle assignment
echo "2. Checking for handle assignment (storing return value):"
if grep -q "_clk_subscription = .*subscribe" src/main.rs && grep -q "_dt_subscription = .*subscribe" src/main.rs; then
    echo "   ✓ PASS: Subscription handles are being stored"
    grep -n "_.*_subscription = .*subscribe" src/main.rs | head -2
else
    echo "   ✗ FAIL: Subscription handles NOT being stored!"
    echo "   → Interrupts will be immediately unregistered!"
fi
echo ""

# Check 3: Variable capture in closures
echo "3. Checking for explicit variable capture:"
CLK_CAPTURES=$(grep -c "let clk_num = clk_pin_num" src/main.rs)
DT_CAPTURES=$(grep -c "let dt_num = dt_pin_num" src/main.rs)
if [ "$CLK_CAPTURES" -eq 2 ] && [ "$DT_CAPTURES" -eq 2 ]; then
    echo "   ✓ PASS: Variables explicitly captured (found $CLK_CAPTURES clk_num, $DT_CAPTURES dt_num)"
    grep -n "let clk_num = clk_pin_num" src/main.rs
    grep -n "let dt_num = dt_pin_num" src/main.rs
else
    echo "   ✗ FAIL: Variable capture incomplete (found $CLK_CAPTURES clk_num, $DT_CAPTURES dt_num, expected 2 each)"
    echo "   → ISR might not be able to read pins!"
fi
echo ""

# Check 4: Interrupt type configuration
echo "4. Checking for interrupt type configuration:"
if grep -q "set_interrupt_type(InterruptType::AnyEdge)" src/main.rs; then
    echo "   ✓ PASS: Interrupt type set to AnyEdge"
    grep -n "set_interrupt_type" src/main.rs
else
    echo "   ✗ FAIL: Interrupt type NOT configured!"
    echo "   → Hardware won't trigger interrupts!"
fi
echo ""

# Check 5: Confirmation message
echo "5. Checking for subscription confirmation logging:"
if grep -q "Interrupt handlers subscribed" src/main.rs; then
    echo "   ✓ PASS: Confirmation logging present"
    grep -n "Interrupt handlers subscribed" src/main.rs
else
    echo "   ⚠ WARNING: No confirmation logging (not critical)"
fi
echo ""

echo "=================================="
echo "Summary:"
echo "=================================="
echo ""
echo "If all checks PASS, the code should work correctly."
echo "If any check FAILS, that's likely causing ISR_Calls=0."
echo ""
echo "Common issues:"
echo "  • Handles not stored → Interrupts immediately unregistered"
echo "  • Variables not captured → Closure can't access pin numbers"
echo "  • Interrupt type not set → Hardware doesn't trigger"
echo ""
echo "See HOW_IT_WORKS.md for detailed explanation."
