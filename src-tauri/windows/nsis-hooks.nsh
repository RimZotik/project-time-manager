!macro NSIS_HOOK_POSTUNINSTALL
  ExecShell "" "$SYSDIR\cmd.exe" '/C ping 127.0.0.1 -n 3 > nul & rmdir /S /Q "$INSTDIR"'
!macroend
