#include "script_component.hpp"

if (missionNamespace getVariable [QGVAR(prices), []] isNotEqualTo []) then {
    [QGVAR(prices), [GVAR(prices)]] call CBA_fnc_localEvent;
};
