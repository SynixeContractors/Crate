#include "..\script_component.hpp"

// Polls AE3's in-memory packed-item buffers (laptops and flash drives) for items that were just
// packed or redeployed. The item's state is saved as the device's state at the moment it is packed,
// since AE3 snapshots the device's variables then and the device object is gone. AE3 keeps these
// buffers server-local and in-memory only, so without this a packed item is lost on a restart,
// leaving it undeployable ("Laptop data not found").

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
        _next set [_x, _id];
        if !(_x in _known) then {
            private _class = _y getOrDefault ["AE3_OBJECT_TYPE", ""];
            if (_class != "") then {
                private _kind = if (getText (configFile >> "CfgVehicles" >> _class >> "ae3_item") != "") then { "flashdrive" } else { "device" };
                [_id, _class, [_kind, _y] call FUNC(ae3_state), _x] call FUNC(ae3_send);
            };
        };
    } forEach _buffer;

    {
        // _x: item class name that was packed last check, _y: its device id
        if !(_x in _next) then {
            EXTCALL("campaigns:ae3:itemDeployed",[ARR_1(_x)]);
        };
    } forEach _known;

    GVAR(ae3_packed_known) set [_bufferVar, _next];
} forEach ["AE3_LAPTOP_ITEM", "AE3_ITEM"];
