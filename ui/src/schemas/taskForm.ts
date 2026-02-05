export const taskFormSchema = {
  type: "card",
  title: "Create New Task",
  children: [
    {
      type: "textField",
      id: "description",
      label: "Description",
      placeholder: "Describe what you want to build...",
      multiline: true,
      rows: 4,
    },
    {
      type: "button",
      id: "submit",
      label: "Submit Task",
      action: "submit",
    },
  ],
};
