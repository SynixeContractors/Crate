#include "..\script_component.hpp"

// Polls AE3's in-memory packed-item buffers (laptops and flash drives) for items that were just
// packed or redeployed, keeping the packed device <-> item mapping in the database. AE3 keeps
// these buffers server-local and in-memory only, so without this a packed item is lost on a
// restart, leaving it undeployable ("Laptop data not found").

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
            EXTCALL("campaigns:ae3:itemPacked",[ARR_2(_x,_id)]);
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
