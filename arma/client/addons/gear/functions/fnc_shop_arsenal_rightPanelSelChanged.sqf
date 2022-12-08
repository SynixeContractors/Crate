#include "script_component.hpp"

_this call ace_arsenal_fnc_onSelChangedRightListnBox;

[{
	params ["_control", "_curSel"];
	[ctrlParent _control, ace_arsenal_currentLeftPanel] call FUNC(shop_arsenal_rightPanelFilled);
}, _this] call CBA_fnc_execNextFrame;
