#include "script_component.hpp"

if !(player diarySubjectExists QGVAR(diary)) then {
    player createDiarySubject [
        QGVAR(diary),
        "Synixe Contractors"
    ];
    GVAR(diaryRecord) = player createDiaryRecord [
        QGVAR(diary),
        [
            "My Account",
            "Loading..."
        ]
    ];
};

private _id = player getVariable [QCVAR(id), "Unknown"];
private _roles = player getVariable [QCVAR(roles), []];

private _roleText = if (_roles isEqualTo []) then {
    ""
} else {
    "Roles:<br/>"
};
{
    _roleText = format ["%1  - %2<br/>", _roleText, _x];
} forEach _roles;

player setDiaryRecordText [[QGVAR(diary), GVAR(diaryRecord)], ["My Account", format [
    "ID: %1<br/><br/>%2",
    _id,
    _roleText
]]];
