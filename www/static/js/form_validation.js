/**
 * see https://www.codementor.io/@robertwozniak/form-validation-system-in-vanilla-javascript-oldy425jc
 */
(function (document) {

    const orderForm = document.getElementById("form-order");
    const validationState = new Set();

    function submitForm() {
        const submitButton = document.getElementById("order-submit");
        submitButton.addEventListener('click', function (event) {

            event.preventDefault();
            // Do some element

            console.log("Print elements !");
        });
    };


    function init() {
        submitForm();
    };


    document.addEventListener('DOMContentLoaded', init);

})(document);