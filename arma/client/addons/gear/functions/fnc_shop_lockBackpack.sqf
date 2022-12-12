#include "script_component.hpp"

params ["_unit", "_backpack"];

private _target = objectParent _backpack;

if (isNull _target) exitWith {false};

if (alive _target && {_target getVariable [QGVAR(shop_open),false]}) exitWith {
    [{
        !isNull (findDisplay 602)
    },
    {
        (findDisplay 602) closeDisplay 0;
        (format ["%1 is in the Shop", name _target]) call CBA_fnc_notify;
    },
    []] call CBA_fnc_waitUntilAndExecute;
};

// return false to open inventory as usual
false
