#include "..\script_component.hpp"

/*
 * Author: SynixeContractors
 * Builds the saved AE3 state of a device or flash drive from its variables.
 *
 * Only AE3's own variables are genuinely part of a device/flash drive's state. Everything else
 * captureState and allVariables also pick up - ACE's cargo/dragging vars, its own interaction menu
 * action tree, CBA's per-object XEH dispatch tables, Crate's own bookkeeping - carries compiled CODE
 * that each mod rebuilds locally on its own; forcing a stale, round-tripped copy back onto a
 * recreated object corrupts it instead ("Script {...} not found")
 *
 * Arguments:
 * 0: _kind <STRING> - "device" or "flashdrive"
 * 1: _values <HASHMAP> - Variables of the device, or of its packed item
 *
 * Return Value:
 * <ARRAY> - [kind, encoded values]
 *
 * Public: No
 */

params ["_kind", "_values"];

private _ae3Values = createHashMap;
{
    if ((toLower _x) find "ae3_" == 0) then {
        _ae3Values set [_x, _values get _x];
    };
} forEach keys _values;

[_kind, [_ae3Values] call FUNC(ae3_encode)]
