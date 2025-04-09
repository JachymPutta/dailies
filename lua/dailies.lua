local M = {}

local binary_path = vim.fn.fnamemodify(
  vim.api.nvim_get_runtime_file("lua/dailies.lua", false)[1], ":h:h"
) .. "/result/bin/dailies"

M.setup = function()
  vim.api.nvim_create_user_command("RunDailies", M.run, {})
end

M.run = function()
  local file = io.popen(binary_path)
  -- This will read all of the output, as always
  local output = file:read('*all')
  file:close()
  local output = output:sub(2, -3)

  vim.schedule(function()
    vim.cmd("vsplit " .. vim.fn.fnameescape(output))
  end)
end

return M
