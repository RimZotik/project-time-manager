!define MUI_FINISHPAGE_SHOWREADME ""
!define MUI_FINISHPAGE_SHOWREADME_TEXT "Удалить установщик после закрытия"
!define MUI_FINISHPAGE_SHOWREADME_FUNCTION DeleteInstallerAfterFinish

Function DeleteInstallerAfterFinish
  ExecShell "" "$SYSDIR\cmd.exe" '/C ping 127.0.0.1 -n 3 > nul & del /F /Q "$EXEPATH"'
FunctionEnd

!macro NSIS_HOOK_POSTUNINSTALL
  RMDir "$INSTDIR"
!macroend
