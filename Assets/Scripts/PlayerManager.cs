using System;
using System.Collections.Generic;
using Unity.Netcode;
using UnityEngine;

public class PlayerManager : NetworkBehaviour
{
    // Network Variables
    private NetworkVariable<Vector3> _networkPosition = new();
    private NetworkVariable<float> _networkRotation = new(writePerm: NetworkVariableWritePermission.Owner);

    private AudioListener audioListener;
    private Camera playerCamera;

    public float mouseSensitivity = 4f;

    [Header("UI Elements")]
    [SerializeField] Sprite grabableSprite;
    [SerializeField] Sprite pickableSprite;
    [SerializeField] Sprite grabSprite;
    [SerializeField] GameObject aimPrefab;
    [SerializeField] GameObject iconPrefab;

    [Header("Grabbing settings")]
    [SerializeField] Transform grabPoint;
    [SerializeField] float maxGrabDistance = 10f;
    [SerializeField] float grabForce = 150f;
    [SerializeField] float grabDamping = 1f;
    private bool lastGrabErrorInitialized = false;
    private Vector3 lastGrabError = Vector3.zero;

    private InteractiveObject interactObject = null;
    private Vector3 grabbedPointRelative;

    private static Dictionary<Type, int> typeMap = new Dictionary<Type, int>();

    private void Awake()
    {
        Debug.Log("PlayerManager Awake");
        audioListener = GetComponentInChildren<AudioListener>();
        playerCamera = GetComponentInChildren<Camera>();

        Cursor.lockState = CursorLockMode.Locked;

        typeMap.Add(typeof(GrabableCup), 0);
    }

    private void Update()
    {
        if (IsServer)
        {
            _networkPosition.Value = transform.position;
        }
        else
        {
            transform.position = _networkPosition.Value;
        }
        if (IsOwner)
        {
            float yRotation = transform.eulerAngles.y + Input.GetAxis("Mouse X") * mouseSensitivity;
            transform.localRotation = Quaternion.Euler(transform.eulerAngles.x, yRotation, 0f);

            float xRotation = playerCamera.transform.localEulerAngles.x - Input.GetAxis("Mouse Y") * mouseSensitivity;
            playerCamera.transform.localRotation = Quaternion.Euler(xRotation, playerCamera.transform.localEulerAngles.y, 0f);
            _networkRotation.Value = transform.rotation.eulerAngles.y;
        }
        else
        {
            transform.eulerAngles = new Vector3(transform.eulerAngles.x, _networkRotation.Value, transform.eulerAngles.z);
        }
    }

    private void FixedUpdate()
    {
        if (!IsOwner) return;

        if (interactObject == null)
        {
            Ray ray = playerCamera.ScreenPointToRay(new Vector3(playerCamera.scaledPixelWidth / 2, playerCamera.scaledPixelHeight / 2, 0));
            int layerMask = 0b1; // Layer 1 is the grabable layer
            if (Physics.Raycast(ray, out RaycastHit hit, maxGrabDistance, layerMask))
            {
                InteractiveObject obj = hit.transform.GetComponentInParent<InteractiveObject>();
                if (obj != null && obj.CanInteract()) {
                    Debug.DrawRay(ray.GetPoint(0), ray.direction * hit.distance, Color.green);
                    aimPrefab.SetActive(false);

                    // Set the icon to the correct sprite
                    if (obj.GetInteractiveType() == InteractiveType.Grab) {
                        iconPrefab.GetComponent<UnityEngine.UI.Image>().sprite = grabableSprite;
                    } else if (obj.GetInteractiveType() == InteractiveType.Pick) {
                        iconPrefab.GetComponent<UnityEngine.UI.Image>().sprite = pickableSprite;
                    }
                    iconPrefab.SetActive(true);

                    if (Input.GetMouseButton(0))
                    {
                        if (obj.Interact()) {
                            interactObject = obj;
                            Debug.Log("Interacting with " + interactObject.name);

                            switch (typeMap[interactObject.GetType()]){
                                case 0: // GrabableCup
                                {
                                    if (GrabCup())
                                    {
                                        Debug.Log("Grabbing " + interactObject.name);
                                        iconPrefab.GetComponent<UnityEngine.UI.Image>().sprite = grabSprite;
                                        grabbedPointRelative = interactObject.transform.InverseTransformPoint(ray.GetPoint(hit.distance));
                                        lastGrabError = Vector3.zero;
                                        lastGrabErrorInitialized = false;
                                    }
                                }; break;
                                default:
                                {
                                    Debug.LogError("Interaction not supported!");
                                }; break;
                            }
                        }
                    }
                }
                else
                {
                    Debug.DrawRay(ray.GetPoint(0), ray.direction * maxGrabDistance, Color.white);
                    aimPrefab.SetActive(true);
                    iconPrefab.SetActive(false);
                }
            }
            else
            {
                Debug.DrawRay(ray.GetPoint(0), ray.direction * maxGrabDistance, Color.white);
                aimPrefab.SetActive(true);
                iconPrefab.SetActive(false);
            }
        }
        else
        {
            switch (typeMap[interactObject.GetType()]){
                case 0: // GrabableCup
                {
                    // PD controller
                    Vector3 error = grabPoint.position - interactObject.transform.TransformPoint(grabbedPointRelative);
                    Vector3 PForce = grabForce * error;
                    Vector3 DForce = lastGrabErrorInitialized ? grabDamping * (error - lastGrabError) / Time.fixedDeltaTime : Vector3.zero;
                    lastGrabError = error;
                    lastGrabErrorInitialized = true;

                    (interactObject as GrabableCup).GetRigidbody().AddForce(PForce + DForce);

                    if (!Input.GetMouseButton(0))
                    {
                        if (interactObject.EndInteraction()) {
                            Debug.Log("End interacting with " + interactObject.name);
                            interactObject = null;
                            if (DropCup()) 
                            {
                                Debug.Log("Dropping item");
                                iconPrefab.GetComponent<UnityEngine.UI.Image>().sprite = grabableSprite;
                            }
                        }
                    }
                }; break;
                default:
                {
                    Debug.LogError("Interaction not supported!");
                }; break;
            }
        }
    }

    private bool GrabCup()
    {
        if (interactObject != null) return false;

        (interactObject as GrabableCup).GetRigidbody().useGravity = false;
        (interactObject as GrabableCup).GetRigidbody().constraints = RigidbodyConstraints.FreezeRotation;
        return true;
    }
    private bool DropCup()
    {
        if (interactObject == null) return false;

        (interactObject as GrabableCup).GetRigidbody().useGravity = true;
        (interactObject as GrabableCup).GetRigidbody().constraints = RigidbodyConstraints.None;
        return true;
    }

    public override void OnNetworkSpawn()
    {
        if (!IsOwner)
        {
            audioListener.enabled = false;
            playerCamera.enabled = false;
        }
    }
}
