#!/bin/sh
# Checks the committed dictionary with the real Hunspell: known words pass, misspellings fail.
set -eu
# Usage: hunspell-smoke.sh [dictionary] [core]. With "core", skip names and loanwords that Alar lacks.
dict="${1:-dict/kn_IN}"
good="ಮರಗಳಲ್ಲಿ ಮನೆಯಲ್ಲಿದೆ ಮಾಡಲಾಗಿದೆ ಮಾಡಿದ್ದಾರೆ ಬಂದರು ಹೋಯಿತು ಎಂಬುದನ್ನು ಅವರಿಗೆ ವಸ್ತುವಿನ ಕಾಡಿನಲ್ಲಿ"
if [ "${2:-}" != "core" ]; then
  good="$good ಬೆಂಗಳೂರಿನಲ್ಲಿ ಬ್ಯಾಂಕ್‌ನಲ್ಲಿ"
fi
bad="ಮರನನ್ನು ಮನೆವನ್ನು ವಿರುದ್ದ ನಿರ್ಧಿಷ್ಟ ಹಾಗು ಬರಿದರು"
rejected=$(printf '%s\n' $good $bad | hunspell -d "$dict" -l)
status=0
for w in $good; do
  if printf '%s\n' "$rejected" | grep -qx "$w"; then echo "rejected a correct word: $w"; status=1; fi
done
for w in $bad; do
  if ! printf '%s\n' "$rejected" | grep -qx "$w"; then echo "accepted a misspelling: $w"; status=1; fi
done
[ "$status" -eq 0 ] && echo "hunspell smoke test passed for $dict"
exit "$status"
