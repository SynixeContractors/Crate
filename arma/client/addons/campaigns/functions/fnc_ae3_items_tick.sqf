#include "..\script_component.hpp"

// Polls AE3's in-memory packed-item buffers (laptops and flash drives) for packed items. Each time an
// item is packed, AE3 writes a new snapshot of the device's variables to its buffer entry, so a
// snapshot that differs from the last one sent is saved as the device's state. AE3 keeps these
// buffers server-local and in-memory only, and never removes a flash drive's entry on redeploy, so the
// key alone can't say whether a pack happened. The packed device <-> item mapping is kept in the
// database too, so a packed item survives a restart ("Laptop data not found" otherwise).

if (isNil "AE3_armaos_fnc_laptop_captureState") exitWith {};

{
    private _bufferVar = _x;
    private _buffer = missionNamespace getVariable [_bufferVar, createHashMap];
    private _known = GVAR(ae3_packed_known) getOrDefault [_bufferVar, createHashMap];
    private _next = createHashMap;

    {
        // _x: item class name, _y: the item's saved variable namespace
        private _id = _y getOrDefault [toLower QGVAR(id), ""];
        if (_id == "") then {
            // Packed before the object ever got a Crate ID, e.g. picked up right after spawning
            EXTFUNC("uuid");
            _id = _ext_res select 0;
            _y set [toLower QGVAR(id), _id];
            _buffer set [_x, _y];
            missionNamespace setVariable [_bufferVar, _buffer, false];
        };
        // _known values are [device id, snapshot last sent]
        _next set [_x, [_id, _y]];
        private _last = _known getOrDefault [_x, []];
        if (_last isEqualTo [] || {!(_y isEqualTo (_last select 1))}) then {
            private _class = _y getOrDefault ["AE3_OBJECT_TYPE", ""];
            if (_class != "") then {
                private _kind = if (getText (configFile >> "CfgVehicles" >> _class >> "ae3_item") != "") then { "flashdrive" } else { "device" };
                [_id, _class, [_kind, _y] call FUNC(ae3_state), _x] call FUNC(ae3_send);
            };
        };
    } forEach _buffer;

    {
        // _x: item class name that was packed last check, _y: [device id, snapshot]
        if !(_x in _next) then {
            EXTCALL("campaigns:ae3:itemDeployed",[ARR_1(_x)]);
        };
    } forEach _known;

    GVAR(ae3_packed_known) set [_bufferVar, _next];
} forEach ["AE3_LAPTOP_ITEM", "AE3_ITEM"];
