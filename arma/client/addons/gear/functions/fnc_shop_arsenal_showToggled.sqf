#include "..\script_component.hpp"

params ["_display", "_state"];

{
    private _ctrl = _display displayCtrl _x;
    _ctrl ctrlShow _state;
    _ctrl ctrlCommit FADE_DELAY;
} forEach [
    IDC_leftRoleFilter,
    IDC_rightRoleFilter
];
